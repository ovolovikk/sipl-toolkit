use std::fs;
use std::path::{Path, PathBuf};

use sipl::interpreter::DEFAULT_ITERATION_LIMIT;
use sipl::parse_tree::{ParseTree, parse_tree, to_text as parse_tree_text};
use sipl::trace::{to_json, to_text, trace};

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

fn initial_state(source: &Path) -> sipl::State {
    let mut state = sipl::State::new();
    let bindings = fs::read_to_string(source.with_extension("in")).unwrap();
    for binding in bindings.split_whitespace() {
        let (name, value) = sipl::State::parse_binding(binding).unwrap();
        state.set(&name, value);
    }
    state
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

fn without_spaces(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn check_leaves_spell_every_node(
    tree: &ParseTree,
    src: &str,
    path: &Path,
    failures: &mut Vec<String>,
) {
    if let ParseTree::Rule { span, children, .. } = tree {
        let leaves = tree.leaves().concat();
        let text = &src[span.start..span.end];
        if without_spaces(&leaves) != without_spaces(text) {
            failures.push(format!(
                "{}\n  `{text}` ≠ листки `{leaves}`",
                path.display()
            ));
        }
        for child in children {
            check_leaves_spell_every_node(child, src, path, failures);
        }
    }
}

#[test]
fn parse_trees_match_golden_and_leaves_spell_the_program() {
    let files = sipl_files("valid");
    let mut failures = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).unwrap();
        let tree = parse_tree(&sipl::parse_program(&src).unwrap(), &src);
        check_golden(
            path,
            "tree",
            parse_tree_text(&tree, &src).trim_end(),
            &mut failures,
        );
        check_leaves_spell_every_node(&tree, &src, path, &mut failures);
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
        let program = sipl::parse_program(&src).unwrap();
        let actual = match sipl::run(&program, initial_state(path)) {
            Ok(state) => state.to_string(),
            Err(err) => err.to_diagnostic().short(&src),
        };
        check_golden(path, "out", &actual, &mut failures);
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn traces_match_golden_text_and_json_ends_with_result() {
    let files = sipl_files("trace");
    assert!(!files.is_empty());
    let mut failures = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).unwrap();
        let program = sipl::parse_program(&src).unwrap();
        let events = trace(&program, initial_state(path), DEFAULT_ITERATION_LIMIT);
        check_golden(
            path,
            "trace",
            to_text(&events, &src).trim_end(),
            &mut failures,
        );

        let json: serde_json::Value =
            serde_json::from_str(&to_json(&events, &src).to_string()).unwrap();
        let last = json["events"].as_array().unwrap().last().unwrap();
        if !matches!(last["event"].as_str(), Some("finished" | "failed")) {
            failures.push(format!("{}\n  остання подія: {last}", path.display()));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
