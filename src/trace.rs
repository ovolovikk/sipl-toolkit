use serde_json::{Map, Value, json};

use crate::ast::{Program, StatementKind};
use crate::interpreter::{RuntimeError, run_observed};
use crate::span::{Span, line_col};
use crate::state::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementTag {
    Assign,
    If,
    While,
    Skip,
}

impl StatementTag {
    pub fn of(kind: &StatementKind) -> Option<StatementTag> {
        match kind {
            StatementKind::Assign { .. } => Some(StatementTag::Assign),
            StatementKind::If { .. } => Some(StatementTag::If),
            StatementKind::While { .. } => Some(StatementTag::While),
            StatementKind::Skip => Some(StatementTag::Skip),
            StatementKind::Sequence(..) | StatementKind::Block(_) => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            StatementTag::Assign => "assign",
            StatementTag::If => "if",
            StatementTag::While => "while",
            StatementTag::Skip => "skip",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    BeforeStatement {
        statement: StatementTag,
        span: Span,
    },
    AfterStatement {
        statement: StatementTag,
        span: Span,
    },
    VariableChanged {
        name: String,
        old: Option<i64>,
        new: i64,
        span: Span,
    },
    ConditionChecked {
        value: bool,
        span: Span,
    },
    Finished {
        state: State,
    },
    Failed {
        error: RuntimeError,
    },
}

pub trait Observer {
    fn on_event(&mut self, event: Event);
}

pub struct Silent;

impl Observer for Silent {
    fn on_event(&mut self, _event: Event) {}
}

impl Observer for Vec<Event> {
    fn on_event(&mut self, event: Event) {
        self.push(event);
    }
}

pub fn trace(program: &Program, state: State, iteration_limit: u64) -> Vec<Event> {
    let mut events = Vec::new();
    let last = match run_observed(program, state, iteration_limit, &mut events) {
        Ok(state) => Event::Finished { state },
        Err(error) => Event::Failed { error },
    };
    events.push(last);
    events
}

pub fn to_json(events: &[Event], src: &str) -> String {
    let events: Vec<Value> = events.iter().map(|event| event_json(event, src)).collect();
    json!({ "events": events }).to_string()
}

fn event_json(event: &Event, src: &str) -> Value {
    match event {
        Event::BeforeStatement { statement, span } => json!({
            "event": "before_statement",
            "statement": statement.name(),
            "span": span_json(*span, src),
        }),
        Event::AfterStatement { statement, span } => json!({
            "event": "after_statement",
            "statement": statement.name(),
            "span": span_json(*span, src),
        }),
        Event::VariableChanged {
            name,
            old,
            new,
            span,
        } => json!({
            "event": "variable_changed",
            "name": name,
            "old": old,
            "new": new,
            "span": span_json(*span, src),
        }),
        Event::ConditionChecked { value, span } => json!({
            "event": "condition_checked",
            "value": value,
            "span": span_json(*span, src),
        }),
        Event::Finished { state } => json!({
            "event": "finished",
            "state": state_json(state),
        }),
        Event::Failed { error } => json!({
            "event": "failed",
            "error": error.code(),
            "message": error.message(),
            "span": span_json(error.span, src),
        }),
    }
}

fn span_json(span: Span, src: &str) -> Value {
    let start = line_col(src, span.start);
    let end = line_col(src, span.end);
    json!({
        "start": { "line": start.line, "column": start.col },
        "end": { "line": end.line, "column": end.col },
    })
}

fn state_json(state: &State) -> Value {
    let bindings: Map<String, Value> = state
        .iter()
        .map(|(name, value)| (name.to_string(), json!(value)))
        .collect();
    Value::Object(bindings)
}

pub fn to_text(events: &[Event], src: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    let mut line = |span: Option<Span>, depth: usize, text: String| {
        let location = span.map_or(String::new(), |span| {
            let lc = line_col(src, span.start);
            format!("{}:{}", lc.line, lc.col)
        });
        out.push_str(&format!("{location:<7}{}{text}\n", "  ".repeat(depth)));
    };
    for event in events {
        match event {
            Event::BeforeStatement { statement, span } => match statement {
                StatementTag::If | StatementTag::While => {
                    line(Some(*span), depth, first_line(src, *span).to_string());
                    depth += 1;
                }
                StatementTag::Skip => line(Some(*span), depth, "skip".to_string()),
                StatementTag::Assign => {}
            },
            Event::AfterStatement { statement, .. } => {
                if matches!(statement, StatementTag::If | StatementTag::While) {
                    depth -= 1;
                }
            }
            Event::VariableChanged {
                name,
                old,
                new,
                span,
            } => {
                let old = old.map_or("—".to_string(), |old| old.to_string());
                let source = first_line(src, *span);
                line(
                    Some(*span),
                    depth,
                    format!("{source:<20}  {name}: {old} → {new}"),
                );
            }
            Event::ConditionChecked { value, span } => {
                line(
                    Some(*span),
                    depth,
                    format!("{} = {value}", first_line(src, *span)),
                );
            }
            Event::Finished { state } => line(None, 0, state.to_string()),
            Event::Failed { error } => {
                line(Some(error.span), 0, format!("помилка: {}", error.message()))
            }
        }
    }
    out
}

fn first_line(src: &str, span: Span) -> &str {
    src[span.start..span.end]
        .lines()
        .next()
        .unwrap_or("")
        .trim_end()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::DEFAULT_ITERATION_LIMIT;
    use crate::parse_program;

    fn events(src: &str, state: State) -> Vec<Event> {
        trace(&parse_program(src).unwrap(), state, DEFAULT_ITERATION_LIMIT)
    }

    fn describe(src: &str, events: &[Event]) -> Vec<String> {
        let text = |span: &Span| &src[span.start..span.end];
        events
            .iter()
            .map(|event| match event {
                Event::BeforeStatement { statement, span } => {
                    format!("before {} `{}`", statement.name(), text(span))
                }
                Event::AfterStatement { statement, span } => {
                    format!("after {} `{}`", statement.name(), text(span))
                }
                Event::VariableChanged { name, old, new, .. } => {
                    format!("{name}: {old:?} -> {new}")
                }
                Event::ConditionChecked { value, span } => format!("`{}` = {value}", text(span)),
                Event::Finished { state } => format!("finished {state}"),
                Event::Failed { error } => format!("failed {}", error.code()),
            })
            .collect()
    }

    #[test]
    fn assignment_and_if() {
        let src = "begin X := 1; if X > 0 then skip else X := 2 end";
        assert_eq!(
            describe(src, &events(src, State::new())),
            [
                "before assign `X := 1`",
                "X: None -> 1",
                "after assign `X := 1`",
                "before if `if X > 0 then skip else X := 2`",
                "`X > 0` = true",
                "before skip `skip`",
                "after skip `skip`",
                "after if `if X > 0 then skip else X := 2`",
                "finished [X ↦ 1]",
            ]
        );
    }

    #[test]
    fn while_checks_condition_before_every_iteration() {
        let src = "begin while X > 0 do X := X - 1 end";
        assert_eq!(
            describe(src, &events(src, State::from([("X", 1)]))),
            [
                "before while `while X > 0 do X := X - 1`",
                "`X > 0` = true",
                "before assign `X := X - 1`",
                "X: Some(1) -> 0",
                "after assign `X := X - 1`",
                "`X > 0` = false",
                "after while `while X > 0 do X := X - 1`",
                "finished [X ↦ 0]",
            ]
        );
    }

    #[test]
    fn failing_statement_has_no_after_event() {
        let src = "begin X := 1; Y := Z end";
        assert_eq!(
            describe(src, &events(src, State::new())),
            [
                "before assign `X := 1`",
                "X: None -> 1",
                "after assign `X := 1`",
                "before assign `Y := Z`",
                "failed undefined_variable",
            ]
        );
    }

    #[test]
    fn iteration_limit_ends_trace() {
        let program = parse_program("begin while 1 > 0 do skip end").unwrap();
        let events = trace(&program, State::new(), 2);
        let last = describe("", &events[events.len() - 1..]);
        assert_eq!(last, ["failed iteration_limit_exceeded"]);
    }

    #[test]
    fn json_is_one_line_with_positions() {
        let src = "begin\n  X := 1\nend";
        assert_eq!(
            to_json(&events(src, State::new()), src),
            concat!(
                r#"{"events":["#,
                r#"{"event":"before_statement","statement":"assign","span":{"start":{"line":2,"column":3},"end":{"line":2,"column":9}}},"#,
                r#"{"event":"variable_changed","name":"X","old":null,"new":1,"span":{"start":{"line":2,"column":3},"end":{"line":2,"column":9}}},"#,
                r#"{"event":"after_statement","statement":"assign","span":{"start":{"line":2,"column":3},"end":{"line":2,"column":9}}},"#,
                r#"{"event":"finished","state":{"X":1}}"#,
                r#"]}"#,
            )
        );
    }

    #[test]
    fn json_failure_and_condition() {
        let src = "begin if X = 0 then skip else skip end";
        let json = to_json(&events(src, State::new()), src);
        assert!(json.ends_with(concat!(
            r#"{"event":"failed","error":"undefined_variable","#,
            r#""message":"змінна `X` не має значення в поточному стані","#,
            r#""span":{"start":{"line":1,"column":10},"end":{"line":1,"column":11}}}]}"#,
        )));
        let src = "begin if 1 = 1 then skip else skip end";
        let json = to_json(&events(src, State::new()), src);
        assert!(json.contains(r#"{"event":"condition_checked","value":true,"#));
    }
}
