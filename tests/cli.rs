use std::process::Command;

use serde_json::Value;

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn sipl(args: &[&str]) -> Outcome {
    let output = Command::new(env!("CARGO_BIN_EXE_sipl"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    Outcome {
        code: output.status.code().unwrap(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

fn sipl_json(args: &[&str]) -> (i32, Value) {
    let mut args = args.to_vec();
    args.push("--json");
    let outcome = sipl(&args);
    assert_eq!(outcome.stdout.lines().count(), 1, "{}", outcome.stdout);
    assert!(outcome.stderr.is_empty(), "{}", outcome.stderr);
    (outcome.code, serde_json::from_str(&outcome.stdout).unwrap())
}

const POWER: &str = "examples/power.sipl";
const TRAILING_SEMICOLON: &str = "tests/fixtures/invalid/04_trailing_semicolon.sipl";

#[test]
fn help_lists_every_command() {
    for flag in ["help", "--help", "-h"] {
        let outcome = sipl(&[flag]);
        assert_eq!(outcome.code, 0);
        for command in ["parse", "check", "run", "trace"] {
            assert!(outcome.stdout.contains(&format!("sipl {command} ")));
        }
    }
}

#[test]
fn parse_prints_ast() {
    let outcome = sipl(&["parse", "examples/gcd.sipl"]);
    assert_eq!(outcome.code, 0);
    assert_eq!(
        outcome.stdout,
        "(while (not (= X Y)) (if (> X Y) (:= X (- X Y)) (:= Y (- Y X))))\n"
    );

    let (code, json) = sipl_json(&["parse", "examples/gcd.sipl"]);
    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(json["ast"]["type"], "program");
    assert_eq!(json["ast"]["body"]["type"], "while");
    assert_eq!(json["ast"]["body"]["condition"]["type"], "not");
    assert_eq!(json["ast"]["body"]["span"]["start"]["line"], 2);
}

#[test]
fn check_reports_syntax() {
    let outcome = sipl(&["check", POWER]);
    assert_eq!(
        (outcome.code, outcome.stdout.as_str()),
        (0, "синтаксис коректний\n")
    );

    let (code, json) = sipl_json(&["check", POWER]);
    assert_eq!((code, json.to_string()), (0, r#"{"ok":true}"#.to_string()));

    let outcome = sipl(&["check", TRAILING_SEMICOLON]);
    assert_eq!(outcome.code, 1);
    assert!(outcome.stdout.is_empty());
    assert!(
        outcome
            .stderr
            .contains(&format!("--> {TRAILING_SEMICOLON}:2:9"))
    );

    let (code, json) = sipl_json(&["check", TRAILING_SEMICOLON]);
    assert_eq!(code, 1);
    assert_eq!(
        json.to_string(),
        concat!(
            r#"{"ok":false,"error":{"kind":"syntax","#,
            r#""message":"зайва `;` перед `end`: у SIPL `;` розділяє оператори, а не завершує їх","#,
            r#""span":{"start":{"line":2,"column":9},"end":{"line":2,"column":10}}}}"#,
        )
    );
}

#[test]
fn syntax_errors_stop_every_command() {
    for command in ["parse", "check", "run", "trace"] {
        let (code, json) = sipl_json(&[command, TRAILING_SEMICOLON]);
        assert_eq!(code, 1, "{command}");
        assert_eq!(json["error"]["kind"], "syntax", "{command}");
    }
}

#[test]
fn run_prints_final_state() {
    let outcome = sipl(&["run", POWER, "X=2", "Y=10"]);
    assert_eq!(outcome.code, 0);
    assert_eq!(outcome.stdout, "[R ↦ 1024, X ↦ 2, Y ↦ 0]\n");

    let (code, json) = sipl_json(&["run", POWER, "X=2", "Y=10"]);
    assert_eq!(code, 0);
    assert_eq!(
        json.to_string(),
        r#"{"ok":true,"state":{"R":1024,"X":2,"Y":0}}"#
    );
}

#[test]
fn run_reports_runtime_errors() {
    let outcome = sipl(&["run", POWER, "X=2"]);
    assert_eq!(outcome.code, 3);
    assert!(outcome.stdout.is_empty());
    assert!(outcome.stderr.contains("змінна `Y` не має значення"));
    assert!(outcome.stderr.contains(&format!("--> {POWER}:3:9")));

    let (code, json) = sipl_json(&["run", POWER, "X=2"]);
    assert_eq!(code, 3);
    assert_eq!(
        json.to_string(),
        concat!(
            r#"{"ok":false,"error":{"kind":"runtime","code":"undefined_variable","#,
            r#""message":"змінна `Y` не має значення в поточному стані","#,
            r#""span":{"start":{"line":3,"column":9},"end":{"line":3,"column":10}}}}"#,
        )
    );

    let (code, json) = sipl_json(&[
        "run",
        "examples/gcd.sipl",
        "X=0",
        "Y=1",
        "--max-iterations",
        "10",
    ]);
    assert_eq!(code, 3);
    assert_eq!(json["error"]["code"], "iteration_limit_exceeded");
}

#[test]
fn trace_prints_events() {
    let outcome = sipl(&["trace", POWER, "X=2", "Y=2"]);
    assert_eq!(outcome.code, 0);
    let golden = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/trace/01_power.trace"),
    )
    .unwrap();
    assert_eq!(outcome.stdout, golden);

    let (code, json) = sipl_json(&["trace", POWER, "X=2", "Y=2"]);
    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.first().unwrap()["event"], "before_statement");
    assert_eq!(events.last().unwrap()["state"]["R"], 4);

    let (code, json) = sipl_json(&["trace", POWER, "X=2"]);
    assert_eq!(code, 3);
    assert_eq!(json["ok"], false);
    assert_eq!(
        json["events"].as_array().unwrap().last().unwrap()["event"],
        "failed"
    );
}

#[test]
fn usage_errors_exit_with_2() {
    let cases: [(&[&str], &str); 9] = [
        (&[], "не вказано команду"),
        (&["frob"], "невідома команда `frob`"),
        (&["run"], "не вказано файл програми"),
        (
            &["parse", POWER, "X=1"],
            "`parse` не приймає початковий стан",
        ),
        (
            &["check", POWER, "--max-iterations", "5"],
            "параметр `--max-iterations` не підходить",
        ),
        (
            &["run", POWER, "--verbose"],
            "параметр `--verbose` не підходить",
        ),
        (&["run", POWER, "X"], "очікувалось `ЗМІННА=ЧИСЛО`"),
        (
            &["run", POWER, "X=two"],
            "`two` не є 64-бітним цілим числом",
        ),
        (
            &["trace", POWER, "--max-iterations", "many"],
            "після `--max-iterations`",
        ),
    ];
    for (args, message) in cases {
        let outcome = sipl(args);
        assert_eq!(outcome.code, 2, "{args:?}");
        assert!(
            outcome.stderr.contains(message),
            "{args:?}: {}",
            outcome.stderr
        );
        assert!(outcome.stderr.contains("використання:"), "{args:?}");

        let (code, json) = sipl_json(args);
        assert_eq!(code, 2, "{args:?}");
        assert_eq!(json["error"]["kind"], "usage", "{args:?}");
        assert!(
            json["error"]["message"].as_str().unwrap().contains(message),
            "{args:?}"
        );
    }
}

#[test]
fn missing_file_exits_with_2() {
    let outcome = sipl(&["run", "missing.sipl"]);
    assert_eq!(outcome.code, 2);
    assert!(
        outcome
            .stderr
            .contains("не вдалося прочитати `missing.sipl`")
    );

    let (code, json) = sipl_json(&["check", "missing.sipl"]);
    assert_eq!(code, 2);
    assert_eq!(json["error"]["kind"], "file");
}

#[test]
fn tree_in_every_format() {
    let outcome = sipl(&["tree", POWER]);
    assert_eq!(outcome.code, 0);
    assert!(
        outcome
            .stdout
            .starts_with("<програма> NP1  1:1  begin…\n├── begin\n")
    );
    assert!(outcome.stdout.ends_with(
        "Листки зліва направо: 1 begin 2 R 3 := 4 1 5 ; 6 while 7 Y 8 > 9 0 10 do 11 begin 12 R 13 := 14 R 15 * 16 X 17 ; 18 Y 19 := 20 Y 21 - 22 1 23 end 24 end\n"
    ));

    let (code, json) = sipl_json(&["tree", POWER]);
    assert_eq!(code, 0);
    assert_eq!(json["ok"], true);
    assert_eq!(json["tree"]["symbol"], "<програма>");
    assert_eq!(json["tree"]["children"][1]["rule"], "NS2");

    let outcome = sipl(&["tree", POWER, "--dot"]);
    assert_eq!(outcome.code, 0);
    assert!(outcome.stdout.starts_with("digraph parse_tree {\n"));

    let outcome = sipl(&["tree", POWER, "--svg"]);
    assert_eq!(outcome.code, 0);
    assert!(outcome.stdout.starts_with("<svg "));

    let (code, json) = sipl_json(&["tree", TRAILING_SEMICOLON]);
    assert_eq!(code, 1);
    assert_eq!(json["error"]["kind"], "syntax");
}

#[test]
fn tree_formats_are_exclusive() {
    for args in [
        &["tree", POWER, "--dot", "--svg"][..],
        &["tree", POWER, "--svg", "--json"],
        &["run", POWER, "--dot"],
    ] {
        let outcome = sipl(args);
        assert_eq!(outcome.code, 2, "{args:?}");
    }
    let outcome = sipl(&["tree", POWER, "--dot", "--svg"]);
    assert!(outcome.stderr.contains("оберіть один формат"));
}
