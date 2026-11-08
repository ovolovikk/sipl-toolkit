use serde_json::{Map, Value, json};

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::interpreter::RuntimeError;
use crate::span::{Span, line_col};
use crate::state::State;

pub fn span_json(span: Span, src: &str) -> Value {
    let start = line_col(src, span.start);
    let end = line_col(src, span.end);
    json!({
        "start": { "line": start.line, "column": start.col },
        "end": { "line": end.line, "column": end.col },
    })
}

pub fn state_json(state: &State) -> Value {
    let bindings: Map<String, Value> = state
        .iter()
        .map(|(name, value)| (name.to_string(), json!(value)))
        .collect();
    Value::Object(bindings)
}

pub fn syntax_error_json(diagnostic: &Diagnostic, src: &str) -> Value {
    json!({
        "kind": "syntax",
        "message": diagnostic.message,
        "span": span_json(diagnostic.span, src),
    })
}

pub fn runtime_error_json(error: &RuntimeError, src: &str) -> Value {
    json!({
        "kind": "runtime",
        "code": error.code(),
        "message": error.message(),
        "span": span_json(error.span, src),
    })
}

pub fn program_json(program: &Program, src: &str) -> Value {
    json!({
        "type": "program",
        "body": statement_json(&program.body, src),
        "span": span_json(program.span, src),
    })
}

fn statement_json(statement: &Statement, src: &str) -> Value {
    let mut node = match &statement.kind {
        StatementKind::Assign { variable, value } => json!({
            "type": "assign",
            "variable": { "name": variable.name, "span": span_json(variable.span, src) },
            "value": arith_json(value, src),
        }),
        StatementKind::Sequence(first, second) => json!({
            "type": "sequence",
            "first": statement_json(first, src),
            "second": statement_json(second, src),
        }),
        StatementKind::If {
            condition,
            then_branch,
            else_branch,
        } => json!({
            "type": "if",
            "condition": bool_json(condition, src),
            "then": statement_json(then_branch, src),
            "else": statement_json(else_branch, src),
        }),
        StatementKind::While { condition, body } => json!({
            "type": "while",
            "condition": bool_json(condition, src),
            "body": statement_json(body, src),
        }),
        StatementKind::Block(inner) => json!({
            "type": "block",
            "body": statement_json(inner, src),
        }),
        StatementKind::Skip => json!({ "type": "skip" }),
    };
    node["span"] = span_json(statement.span, src);
    node
}

fn arith_json(expr: &ArithExpr, src: &str) -> Value {
    let mut node = match &expr.kind {
        ArithExprKind::Number(value) => json!({ "type": "number", "value": value }),
        ArithExprKind::Variable(name) => json!({ "type": "variable", "name": name }),
        ArithExprKind::Binary { op, left, right } => json!({
            "type": "binary",
            "op": op.symbol(),
            "left": arith_json(left, src),
            "right": arith_json(right, src),
        }),
    };
    node["span"] = span_json(expr.span, src);
    node
}

fn bool_json(condition: &BoolExpr, src: &str) -> Value {
    let mut node = match &condition.kind {
        BoolExprKind::Compare { op, left, right } => json!({
            "type": "compare",
            "op": op.symbol(),
            "left": arith_json(left, src),
            "right": arith_json(right, src),
        }),
        BoolExprKind::Or(left, right) => json!({
            "type": "or",
            "left": bool_json(left, src),
            "right": bool_json(right, src),
        }),
        BoolExprKind::Not(operand) => json!({
            "type": "not",
            "operand": bool_json(operand, src),
        }),
    };
    node["span"] = span_json(condition.span, src);
    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_program;

    fn ast(src: &str) -> String {
        program_json(&parse_program(src).unwrap(), src).to_string()
    }

    fn span(start_col: usize, end_col: usize) -> String {
        format!(
            r#"{{"start":{{"line":1,"column":{start_col}}},"end":{{"line":1,"column":{end_col}}}}}"#
        )
    }

    #[test]
    fn assignment_with_binary_expression() {
        let src = "begin X := Y + 1 end";
        let expected = format!(
            concat!(
                r#"{{"type":"program","body":{{"type":"assign","#,
                r#""variable":{{"name":"X","span":{x}}},"#,
                r#""value":{{"type":"binary","op":"+","#,
                r#""left":{{"type":"variable","name":"Y","span":{y}}},"#,
                r#""right":{{"type":"number","value":1,"span":{one}}},"#,
                r#""span":{sum}}},"span":{assign}}},"span":{program}}}"#,
            ),
            x = span(7, 8),
            y = span(12, 13),
            one = span(16, 17),
            sum = span(12, 17),
            assign = span(7, 17),
            program = span(1, 21),
        );
        assert_eq!(ast(src), expected);
    }

    #[test]
    fn every_node_type_appears() {
        let src = "begin skip; begin if ¬(X = 1) ∨ X > 2 then skip else while X > 0 do X := X - 1 * 2 end end";
        let json = ast(src);
        for node in [
            "program", "sequence", "skip", "block", "if", "not", "or", "compare", "while",
            "assign", "binary", "variable", "number",
        ] {
            assert!(json.contains(&format!(r#""type":"{node}""#)), "{node}");
        }
    }

    #[test]
    fn errors() {
        let src = "begin X := Y end";
        let syntax = Diagnostic::new("повідомлення", Span::new(6, 7));
        assert_eq!(
            syntax_error_json(&syntax, src).to_string(),
            format!(
                r#"{{"kind":"syntax","message":"повідомлення","span":{}}}"#,
                span(7, 8)
            )
        );
        let runtime = crate::run(&parse_program(src).unwrap(), State::new()).unwrap_err();
        assert_eq!(
            runtime_error_json(&runtime, src).to_string(),
            format!(
                r#"{{"kind":"runtime","code":"undefined_variable","message":"змінна `Y` не має значення в поточному стані","span":{}}}"#,
                span(12, 13)
            )
        );
    }
}
