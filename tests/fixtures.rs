use std::fs;
use std::path::{Path, PathBuf};

fn sipl_files(dir: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(dir);
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sipl"))
        .collect();
    files.sort();
    files
}

fn check_golden(source: &Path, ext: &str, actual: &str, failures: &mut Vec<String>) {
    let golden = source.with_extension(ext);
    if std::env::var_os("SIPL_BLESS").is_some() {
        fs::write(&golden, format!("{actual}\n")).unwrap();
        return;
    }
    let expected = fs::read_to_string(&golden).unwrap_or_default();
    if expected.trim_end() != actual {
        failures.push(format!(
            "{}\n  очікувалось: {}\n  отримано:    {actual}",
            source.display(),
            expected.trim_end()
        ));
    }
}

#[test]
fn valid_programs_match_golden_ast() {
    let files = sipl_files("valid");
    assert!(files.len() >= 10);
    let mut failures = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).unwrap();
        match sipl::parse_program(&src) {
            Ok(program) => check_golden(path, "ast", &program.to_string(), &mut failures),
            Err(err) => failures.push(format!(
                "{}\n  неочікувана помилка: {}",
                path.display(),
                err.short(&src)
            )),
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn invalid_programs_match_golden_errors() {
    let files = sipl_files("invalid");
    assert!(files.len() >= 10);
    let mut failures = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).unwrap();
        match sipl::parse_program(&src) {
            Ok(program) => failures.push(format!(
                "{}\n  розібралась без помилки: {program}",
                path.display()
            )),
            Err(err) => check_golden(path, "err", &err.short(&src), &mut failures),
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn programs_run_to_golden_results() {
    let files = sipl_files("run");
    assert!(files.len() >= 10);
    let mut failures = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).unwrap();
        let mut state = sipl::State::new();
        for binding in fs::read_to_string(path.with_extension("in"))
            .unwrap()
            .split_whitespace()
        {
            let (name, value) = sipl::State::parse_binding(binding).unwrap();
            state.set(&name, value);
        }
        let program = sipl::parse_program(&src).unwrap();
        let actual = match sipl::run(&program, state) {
            Ok(state) => state.to_string(),
            Err(err) => err.to_diagnostic().short(&src),
        };
        check_golden(path, "out", &actual, &mut failures);
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
