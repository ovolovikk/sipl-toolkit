use std::process::ExitCode;

use serde_json::{Value, json};
use sipl::diagnostic::Diagnostic;
use sipl::interpreter::{DEFAULT_ITERATION_LIMIT, RuntimeError};
use sipl::json::{program_json, runtime_error_json, state_json, syntax_error_json};
use sipl::parse_tree::{ParseTree, parse_tree, to_dot, to_svg};
use sipl::trace::{finished, to_json, to_text, trace};
use sipl::{State, parse_program, run_with_iteration_limit};

const USAGE: &str = "використання:
  sipl parse <файл.sipl> [--json]
  sipl check <файл.sipl> [--json]
  sipl run <файл.sipl> [ЗМІННА=ЧИСЛО ...] [--max-iterations N] [--json]
  sipl trace <файл.sipl> [ЗМІННА=ЧИСЛО ...] [--max-iterations N] [--json]
  sipl tree <файл.sipl> [--json | --dot | --svg]
  sipl help";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    Parse,
    Check,
    Run,
    Trace,
    Tree,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Drawing {
    Dot,
    Svg,
}

struct Request {
    command: Command,
    path: String,
    state: State,
    iteration_limit: u64,
    drawing: Option<Drawing>,
}

struct Source {
    path: String,
    text: String,
}

enum Output {
    Ast(sipl::ast::Program),
    Checked,
    State(State),
    Trace(Vec<sipl::trace::Event>),
    Tree(ParseTree, Option<Drawing>),
}

enum Failure {
    Usage(String),
    File(String),
    Syntax(Diagnostic),
    Runtime(RuntimeError),
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if matches!(
        args.first().map(String::as_str),
        Some("help" | "--help" | "-h")
    ) {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let json = args.iter().any(|arg| arg == "--json");
    let request = match parse_request(&args) {
        Ok(request) => request,
        Err(failure) => return report_failure(&failure, None, json),
    };
    let source = match std::fs::read_to_string(&request.path) {
        Ok(text) => Source {
            path: request.path.clone(),
            text,
        },
        Err(err) => {
            let message = format!("не вдалося прочитати `{}`: {err}", request.path);
            return report_failure(&Failure::File(message), None, json);
        }
    };
    match execute(request, &source.text) {
        Ok(output) => report_output(&output, &source, json),
        Err(failure) => report_failure(&failure, Some(&source), json),
    }
}

fn parse_request(args: &[String]) -> Result<Request, Failure> {
    let usage = |message: &str| Failure::Usage(message.to_string());
    let json = args.iter().any(|arg| arg == "--json");
    let args: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|&arg| arg != "--json")
        .collect();
    let command = match args.first().copied() {
        Some("parse") => Command::Parse,
        Some("check") => Command::Check,
        Some("run") => Command::Run,
        Some("trace") => Command::Trace,
        Some("tree") => Command::Tree,
        Some(other) => return Err(usage(&format!("невідома команда `{other}`"))),
        None => return Err(usage("не вказано команду")),
    };
    let path = match args.get(1) {
        Some(path) if !path.starts_with("--") => path.to_string(),
        _ => return Err(usage("не вказано файл програми")),
    };
    let executes = matches!(command, Command::Run | Command::Trace);
    let mut request = Request {
        command,
        path,
        state: State::new(),
        iteration_limit: DEFAULT_ITERATION_LIMIT,
        drawing: None,
    };
    let mut rest = args[2..].iter();
    while let Some(&arg) = rest.next() {
        match arg {
            "--max-iterations" if executes => {
                request.iteration_limit = rest
                    .next()
                    .and_then(|value| value.parse().ok())
                    .ok_or_else(|| usage("після `--max-iterations` очікувалось ціле число"))?;
            }
            "--dot" | "--svg" if command == Command::Tree => {
                if json || request.drawing.is_some() {
                    return Err(usage("оберіть один формат: `--json`, `--dot` або `--svg`"));
                }
                request.drawing = Some(if arg == "--dot" {
                    Drawing::Dot
                } else {
                    Drawing::Svg
                });
            }
            option if option.starts_with("--") => {
                return Err(usage(&format!(
                    "параметр `{option}` не підходить до цієї команди"
                )));
            }
            _ if !executes => {
                return Err(usage(&format!("`{}` не приймає початковий стан", args[0])));
            }
            binding => {
                let (name, value) = State::parse_binding(binding).map_err(Failure::Usage)?;
                request.state.set(&name, value);
            }
        }
    }
    Ok(request)
}

fn execute(request: Request, src: &str) -> Result<Output, Failure> {
    let program = parse_program(src).map_err(Failure::Syntax)?;
    match request.command {
        Command::Parse => Ok(Output::Ast(program)),
        Command::Check => Ok(Output::Checked),
        Command::Run => run_with_iteration_limit(&program, request.state, request.iteration_limit)
            .map(Output::State)
            .map_err(Failure::Runtime),
        Command::Trace => Ok(Output::Trace(trace(
            &program,
            request.state,
            request.iteration_limit,
        ))),
        Command::Tree => Ok(Output::Tree(parse_tree(&program, src), request.drawing)),
    }
}

fn report_output(output: &Output, source: &Source, json: bool) -> ExitCode {
    let src = &source.text;
    if json {
        let value = match output {
            Output::Ast(program) => json!({ "ok": true, "ast": program_json(program, src) }),
            Output::Checked => json!({ "ok": true }),
            Output::State(state) => json!({ "ok": true, "state": state_json(state) }),
            Output::Trace(events) => to_json(events, src),
            Output::Tree(tree, _) => {
                json!({ "ok": true, "tree": sipl::parse_tree::to_json(tree, src) })
            }
        };
        println!("{value}");
    } else {
        match output {
            Output::Ast(program) => println!("{program}"),
            Output::Checked => println!("синтаксис коректний"),
            Output::State(state) => println!("{state}"),
            Output::Trace(events) => print!("{}", to_text(events, src)),
            Output::Tree(tree, None) => print!("{}", sipl::parse_tree::to_text(tree, src)),
            Output::Tree(tree, Some(Drawing::Dot)) => print!("{}", to_dot(tree, src)),
            Output::Tree(tree, Some(Drawing::Svg)) => print!("{}", to_svg(tree, src)),
        }
    }
    match output {
        Output::Trace(events) if !finished(events) => ExitCode::from(3),
        _ => ExitCode::SUCCESS,
    }
}

fn report_failure(failure: &Failure, source: Option<&Source>, json: bool) -> ExitCode {
    let src = source.map_or("", |source| source.text.as_str());
    let path = source.map_or("", |source| source.path.as_str());
    if json {
        let error: Value = match failure {
            Failure::Usage(message) => json!({ "kind": "usage", "message": message }),
            Failure::File(message) => json!({ "kind": "file", "message": message }),
            Failure::Syntax(diagnostic) => syntax_error_json(diagnostic, src),
            Failure::Runtime(error) => runtime_error_json(error, src),
        };
        println!("{}", json!({ "ok": false, "error": error }));
    } else {
        match failure {
            Failure::Usage(message) => eprintln!("помилка: {message}\n{USAGE}"),
            Failure::File(message) => eprintln!("помилка: {message}"),
            Failure::Syntax(diagnostic) => eprint!("{}", diagnostic.render(src, path)),
            Failure::Runtime(error) => eprint!("{}", error.to_diagnostic().render(src, path)),
        }
    }
    ExitCode::from(match failure {
        Failure::Syntax(_) => 1,
        Failure::Usage(_) | Failure::File(_) => 2,
        Failure::Runtime(_) => 3,
    })
}
