use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("використання: sipl <файл.sipl>");
        return ExitCode::from(2);
    };
    let src = match std::fs::read_to_string(&path) {
        Ok(src) => src,
        Err(err) => {
            eprintln!("не вдалося прочитати `{path}`: {err}");
            return ExitCode::from(2);
        }
    };
    match sipl::parse_program(&src) {
        Ok(program) => {
            println!("{program}");
            ExitCode::SUCCESS
        }
        Err(diagnostic) => {
            eprint!("{}", diagnostic.render(&src, &path));
            ExitCode::FAILURE
        }
    }
}
