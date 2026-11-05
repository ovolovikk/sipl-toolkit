use std::process::ExitCode;

use sipl::ast::Program;
use sipl::interpreter::DEFAULT_ITERATION_LIMIT;
use sipl::{State, parse_program, run_with_iteration_limit};

const USAGE: &str = "використання:
  sipl parse <файл.sipl>
  sipl run <файл.sipl> [ЗМІННА=ЧИСЛО ...] [--max-iterations N]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("parse") if args.len() == 2 => parse_command(&args[1]),
        Some("run") if args.len() >= 2 => run_command(&args[1], &args[2..]),
        _ => Err(usage_error("")),
    };
    result.unwrap_or_else(|code| code)
}

fn parse_command(path: &str) -> Result<ExitCode, ExitCode> {
    let (_, program) = load(path)?;
    println!("{program}");
    Ok(ExitCode::SUCCESS)
}

fn run_command(path: &str, args: &[String]) -> Result<ExitCode, ExitCode> {
    let mut state = State::new();
    let mut iteration_limit = DEFAULT_ITERATION_LIMIT;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == "--max-iterations" {
            iteration_limit = rest
                .next()
                .and_then(|value| value.parse().ok())
                .ok_or_else(|| {
                    usage_error("після `--max-iterations` очікувалось додатне ціле число")
                })?;
        } else {
            let (name, value) =
                State::parse_binding(arg).map_err(|message| usage_error(&message))?;
            state.set(&name, value);
        }
    }

    let (src, program) = load(path)?;
    match run_with_iteration_limit(&program, state, iteration_limit) {
        Ok(state) => {
            println!("{state}");
            Ok(ExitCode::SUCCESS)
        }
        Err(err) => {
            eprint!("{}", err.to_diagnostic().render(&src, path));
            Err(ExitCode::FAILURE)
        }
    }
}

fn load(path: &str) -> Result<(String, Program), ExitCode> {
    let src = std::fs::read_to_string(path).map_err(|err| {
        eprintln!("не вдалося прочитати `{path}`: {err}");
        ExitCode::from(2)
    })?;
    match parse_program(&src) {
        Ok(program) => Ok((src, program)),
        Err(diagnostic) => {
            eprint!("{}", diagnostic.render(&src, path));
            Err(ExitCode::FAILURE)
        }
    }
}

fn usage_error(message: &str) -> ExitCode {
    if !message.is_empty() {
        eprintln!("помилка: {message}");
    }
    eprintln!("{USAGE}");
    ExitCode::from(2)
}
