use serde_json::{Value, json};

use crate::ast::*;
use crate::json::span_json;
use crate::lexer::{Token, TokenKind, tokenize};
use crate::span::{Span, line_col};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Program,
    Statement,
    Expression,
    Condition,
    Variable,
    Number,
}

impl Symbol {
    pub fn name(self) -> &'static str {
        match self {
            Symbol::Program => "<програма>",
            Symbol::Statement => "<оператор>",
            Symbol::Expression => "<вираз>",
            Symbol::Condition => "<умова>",
            Symbol::Variable => "<змінна>",
            Symbol::Number => "<число>",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseTree {
    Rule {
        symbol: Symbol,
        rule: &'static str,
        span: Span,
        children: Vec<ParseTree>,
    },
    Token {
        text: String,
        span: Span,
    },
}

impl ParseTree {
    pub fn span(&self) -> Span {
        match self {
            ParseTree::Rule { span, .. } | ParseTree::Token { span, .. } => *span,
        }
    }

    pub fn leaves(&self) -> Vec<&str> {
        match self {
            ParseTree::Token { text, .. } => vec![text.as_str()],
            ParseTree::Rule { children, .. } => {
                children.iter().flat_map(ParseTree::leaves).collect()
            }
        }
    }
}

pub fn parse_tree(program: &Program, src: &str) -> ParseTree {
    let tokens = tokenize(src).expect("program was parsed from this source");
    let builder = Builder { src, tokens };
    builder.program(program)
}

struct Builder<'a> {
    src: &'a str,
    tokens: Vec<Token>,
}

impl Builder<'_> {
    fn index_at(&self, offset: usize) -> usize {
        self.tokens
            .iter()
            .position(|token| token.span.start >= offset)
            .unwrap_or(self.tokens.len() - 1)
    }

    fn leaf(&self, index: usize) -> ParseTree {
        let span = self.tokens[index].span;
        self.leaf_span(span)
    }

    fn leaf_span(&self, span: Span) -> ParseTree {
        ParseTree::Token {
            text: self.src[span.start..span.end].to_string(),
            span,
        }
    }

    fn leaf_at(&self, offset: usize) -> ParseTree {
        self.leaf(self.index_at(offset))
    }

    fn leaf_before(&self, end: usize) -> ParseTree {
        let index = self
            .tokens
            .iter()
            .rposition(|token| token.kind != TokenKind::Eof && token.span.end <= end)
            .unwrap_or(0);
        self.leaf(index)
    }

    fn unwrap_parens(&self, span: Span) -> Option<Span> {
        let open = self.index_at(span.start);
        if self.tokens[open].kind != TokenKind::LeftParen {
            return None;
        }
        let mut depth = 0;
        for close in open..self.tokens.len() {
            match self.tokens[close].kind {
                TokenKind::LeftParen => depth += 1,
                TokenKind::RightParen => {
                    depth -= 1;
                    if depth == 0 {
                        if self.tokens[close].span.end != span.end {
                            return None;
                        }
                        let inner = self.tokens[open + 1].span.start;
                        return Some(Span::new(inner, self.tokens[close - 1].span.end));
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn rule(symbol: Symbol, rule: &'static str, span: Span, children: Vec<ParseTree>) -> ParseTree {
        ParseTree::Rule {
            symbol,
            rule,
            span,
            children,
        }
    }

    fn program(&self, program: &Program) -> ParseTree {
        Self::rule(
            Symbol::Program,
            "NP1",
            program.span,
            vec![
                self.leaf_at(program.span.start),
                self.statement(&program.body),
                self.leaf_before(program.span.end),
            ],
        )
    }

    fn statement(&self, statement: &Statement) -> ParseTree {
        let span = statement.span;
        let (rule, children) = match &statement.kind {
            StatementKind::Assign { variable, value } => (
                "NS1",
                vec![
                    self.variable(variable.span),
                    self.leaf_at(variable.span.end),
                    self.expression(value, value.span),
                ],
            ),
            StatementKind::Sequence(first, second) => (
                "NS2",
                vec![
                    self.statement(first),
                    self.leaf_at(first.span.end),
                    self.statement(second),
                ],
            ),
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => (
                "NS3",
                vec![
                    self.leaf_at(span.start),
                    self.condition(condition, condition.span),
                    self.leaf_at(condition.span.end),
                    self.statement(then_branch),
                    self.leaf_at(then_branch.span.end),
                    self.statement(else_branch),
                ],
            ),
            StatementKind::While { condition, body } => (
                "NS4",
                vec![
                    self.leaf_at(span.start),
                    self.condition(condition, condition.span),
                    self.leaf_at(condition.span.end),
                    self.statement(body),
                ],
            ),
            StatementKind::Block(inner) => (
                "NS5",
                vec![
                    self.leaf_at(span.start),
                    self.statement(inner),
                    self.leaf_before(span.end),
                ],
            ),
            StatementKind::Skip => ("NS6", vec![self.leaf_at(span.start)]),
        };
        Self::rule(Symbol::Statement, rule, span, children)
    }

    fn expression(&self, expr: &ArithExpr, span: Span) -> ParseTree {
        if let Some(inner) = self.unwrap_parens(span) {
            return Self::rule(
                Symbol::Expression,
                "NA6",
                span,
                vec![
                    self.leaf_at(span.start),
                    self.expression(expr, inner),
                    self.leaf_before(span.end),
                ],
            );
        }
        let (rule, children) = match &expr.kind {
            ArithExprKind::Number(_) => (
                "NA1",
                vec![Self::rule(
                    Symbol::Number,
                    "NN",
                    span,
                    vec![self.leaf_span(span)],
                )],
            ),
            ArithExprKind::Variable(_) => ("NA2", vec![self.variable(span)]),
            ArithExprKind::Binary { op, left, right } => (
                match op {
                    ArithOp::Add => "NA3",
                    ArithOp::Subtract => "NA4",
                    ArithOp::Multiply => "NA5",
                },
                vec![
                    self.expression(left, left.span),
                    self.leaf_at(left.span.end),
                    self.expression(right, right.span),
                ],
            ),
        };
        Self::rule(Symbol::Expression, rule, span, children)
    }

    fn condition(&self, condition: &BoolExpr, span: Span) -> ParseTree {
        if let Some(inner) = self.unwrap_parens(span) {
            return Self::rule(
                Symbol::Condition,
                "NB5",
                span,
                vec![
                    self.leaf_at(span.start),
                    self.condition(condition, inner),
                    self.leaf_before(span.end),
                ],
            );
        }
        let (rule, children) = match &condition.kind {
            BoolExprKind::Compare { op, left, right } => (
                match op {
                    ComparisonOp::Equal => "NB1",
                    ComparisonOp::Greater => "NB2",
                },
                vec![
                    self.expression(left, left.span),
                    self.leaf_at(left.span.end),
                    self.expression(right, right.span),
                ],
            ),
            BoolExprKind::Or(left, right) => (
                "NB3",
                vec![
                    self.condition(left, left.span),
                    self.leaf_at(left.span.end),
                    self.condition(right, right.span),
                ],
            ),
            BoolExprKind::Not(operand) => (
                "NB4",
                vec![
                    self.leaf_at(span.start),
                    self.condition(operand, operand.span),
                ],
            ),
        };
        Self::rule(Symbol::Condition, rule, span, children)
    }

    fn variable(&self, span: Span) -> ParseTree {
        Self::rule(Symbol::Variable, "NV", span, vec![self.leaf_span(span)])
    }
}

fn fragment(src: &str, span: Span) -> String {
    let text = &src[span.start..span.end];
    let first = text.lines().next().unwrap_or("").trim_end();
    let short: String = first.chars().take(32).collect();
    if short.len() < text.trim_end().len() {
        format!("{short}…")
    } else {
        short
    }
}

fn position(src: &str, span: Span) -> String {
    let lc = line_col(src, span.start);
    format!("{}:{}", lc.line, lc.col)
}

pub fn to_text(tree: &ParseTree, src: &str) -> String {
    let mut out = String::new();
    write_text(tree, src, "", "", &mut out);
    let leaves: Vec<String> = tree
        .leaves()
        .iter()
        .enumerate()
        .map(|(i, leaf)| format!("{} {leaf}", i + 1))
        .collect();
    out.push_str(&format!("\nЛистки зліва направо: {}\n", leaves.join(" ")));
    out
}

fn write_text(tree: &ParseTree, src: &str, first: &str, rest: &str, out: &mut String) {
    match tree {
        ParseTree::Token { text, .. } => out.push_str(&format!("{first}{text}\n")),
        ParseTree::Rule {
            symbol,
            rule,
            span,
            children,
        } => {
            out.push_str(&format!(
                "{first}{} {rule}  {}  {}\n",
                symbol.name(),
                position(src, *span),
                fragment(src, *span)
            ));
            for (i, child) in children.iter().enumerate() {
                let last = i + 1 == children.len();
                let (branch, indent) = if last {
                    ("└── ", "    ")
                } else {
                    ("├── ", "│   ")
                };
                write_text(
                    child,
                    src,
                    &format!("{rest}{branch}"),
                    &format!("{rest}{indent}"),
                    out,
                );
            }
        }
    }
}

pub fn to_json(tree: &ParseTree, src: &str) -> Value {
    match tree {
        ParseTree::Token { text, span } => json!({
            "token": text,
            "span": span_json(*span, src),
        }),
        ParseTree::Rule {
            symbol,
            rule,
            span,
            children,
        } => json!({
            "symbol": symbol.name(),
            "rule": rule,
            "span": span_json(*span, src),
            "children": children.iter().map(|child| to_json(child, src)).collect::<Vec<_>>(),
        }),
    }
}

fn escape_dot(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn to_dot(tree: &ParseTree, src: &str) -> String {
    let mut out =
        String::from("digraph parse_tree {\n  ordering=out;\n  node [fontname=\"Helvetica\"];\n");
    let mut next = 0;
    write_dot(tree, src, &mut next, &mut out);
    out.push_str("}\n");
    out
}

fn write_dot(tree: &ParseTree, src: &str, next: &mut usize, out: &mut String) -> usize {
    let id = *next;
    *next += 1;
    let tooltip = escape_dot(&format!(
        "{}  {}",
        position(src, tree.span()),
        fragment(src, tree.span())
    ));
    match tree {
        ParseTree::Token { text, .. } => out.push_str(&format!(
            "  n{id} [label=\"{}\", shape=plaintext, tooltip=\"{tooltip}\"];\n",
            escape_dot(text)
        )),
        ParseTree::Rule {
            symbol,
            rule,
            children,
            ..
        } => {
            out.push_str(&format!(
                "  n{id} [label=\"{}\\n{rule}\", shape=box, style=rounded, tooltip=\"{tooltip}\"];\n",
                escape_dot(symbol.name())
            ));
            for child in children {
                let child_id = write_dot(child, src, next, out);
                out.push_str(&format!("  n{id} -> n{child_id};\n"));
            }
        }
    }
    id
}

const SVG_MARGIN: f64 = 20.0;
const SVG_LEVEL: f64 = 64.0;
const SVG_GAP: f64 = 10.0;
const SVG_RULE_HEIGHT: f64 = 34.0;
const SVG_LEAF_HEIGHT: f64 = 22.0;

struct Layout<'t> {
    node: &'t ParseTree,
    width: f64,
    children: Vec<Layout<'t>>,
}

fn own_width(node: &ParseTree) -> f64 {
    match node {
        ParseTree::Token { text, .. } => text.chars().count() as f64 * 8.5 + 12.0,
        ParseTree::Rule { symbol, rule, .. } => {
            symbol.name().chars().count().max(rule.len()) as f64 * 7.5 + 16.0
        }
    }
}

fn measure(node: &ParseTree) -> Layout<'_> {
    let children: Vec<Layout> = match node {
        ParseTree::Token { .. } => Vec::new(),
        ParseTree::Rule { children, .. } => children.iter().map(measure).collect(),
    };
    let children_width = children.iter().map(|child| child.width).sum::<f64>()
        + SVG_GAP * children.len().saturating_sub(1) as f64;
    Layout {
        width: own_width(node).max(children_width),
        node,
        children,
    }
}

fn depth(node: &ParseTree) -> usize {
    match node {
        ParseTree::Token { .. } => 0,
        ParseTree::Rule { children, .. } => 1 + children.iter().map(depth).max().unwrap_or(0),
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

struct SvgWriter<'s> {
    src: &'s str,
    edges: String,
    nodes: String,
    leaf_number: usize,
}

impl SvgWriter<'_> {
    fn place(&mut self, layout: &Layout, left: f64, level: usize) -> f64 {
        let center = left + layout.width / 2.0;
        let top = SVG_MARGIN + level as f64 * SVG_LEVEL;
        let title = escape_xml(&format!(
            "{}  {}",
            position(self.src, layout.node.span()),
            fragment(self.src, layout.node.span())
        ));
        match layout.node {
            ParseTree::Token { text, .. } => {
                self.leaf_number += 1;
                self.nodes.push_str(&format!(
                    concat!(
                        r#"<g class="leaf"><title>{title}</title>"#,
                        r#"<text x="{x:.1}" y="{y:.1}">{text}</text>"#,
                        r#"<text class="order" x="{x:.1}" y="{order_y:.1}">{n}</text></g>"#,
                        "\n"
                    ),
                    title = title,
                    x = center,
                    y = top + 16.0,
                    text = escape_xml(text),
                    order_y = top + SVG_LEAF_HEIGHT + 10.0,
                    n = self.leaf_number,
                ));
            }
            ParseTree::Rule { symbol, rule, .. } => {
                let width = own_width(layout.node);
                self.nodes.push_str(&format!(
                    concat!(
                        r#"<g class="rule"><title>{title}</title>"#,
                        r#"<rect x="{x:.1}" y="{y:.1}" width="{w:.1}" height="{h:.1}" rx="6"/>"#,
                        r#"<text x="{cx:.1}" y="{symbol_y:.1}">{symbol}</text>"#,
                        r#"<text class="name" x="{cx:.1}" y="{rule_y:.1}">{rule}</text></g>"#,
                        "\n"
                    ),
                    title = title,
                    x = center - width / 2.0,
                    y = top,
                    w = width,
                    h = SVG_RULE_HEIGHT,
                    cx = center,
                    symbol_y = top + 14.0,
                    symbol = escape_xml(symbol.name()),
                    rule_y = top + 28.0,
                    rule = rule,
                ));
                let total = layout.children.iter().map(|child| child.width).sum::<f64>()
                    + SVG_GAP * layout.children.len().saturating_sub(1) as f64;
                let mut child_left = left + (layout.width - total) / 2.0;
                for child in &layout.children {
                    let child_center = self.place(child, child_left, level + 1);
                    self.edges.push_str(&format!(
                        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\"/>\n",
                        center,
                        top + SVG_RULE_HEIGHT,
                        child_center,
                        top + SVG_LEVEL
                    ));
                    child_left += child.width + SVG_GAP;
                }
            }
        }
        center
    }
}

pub fn to_svg(tree: &ParseTree, src: &str) -> String {
    let layout = measure(tree);
    let width = layout.width + 2.0 * SVG_MARGIN;
    let height = 2.0 * SVG_MARGIN + depth(tree) as f64 * SVG_LEVEL + SVG_LEAF_HEIGHT;
    let mut writer = SvgWriter {
        src,
        edges: String::new(),
        nodes: String::new(),
        leaf_number: 0,
    };
    writer.place(&layout, SVG_MARGIN, 0);
    format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="{h:.0}" viewBox="0 0 {w:.0} {h:.0}" font-family="Helvetica, Arial, sans-serif" text-anchor="middle">"#,
            "\n<style>",
            ".rule rect{{fill:#eef3f8;stroke:#4b6584}}",
            ".rule text{{fill:#1e2a38;font-size:12px}}",
            ".rule .name{{fill:#5b6b7c;font-size:10px}}",
            ".leaf text{{fill:#a33d2a;font-family:Menlo,Consolas,monospace;font-size:14px}}",
            ".leaf .order{{fill:#8a96a3;font-family:Helvetica,Arial,sans-serif;font-size:10px}}",
            "line{{stroke:#9aa5b1}}",
            "</style>\n",
            r##"<rect width="100%" height="100%" fill="#ffffff"/>"##,
            "\n{edges}{nodes}</svg>\n"
        ),
        w = width,
        h = height,
        edges = writer.edges,
        nodes = writer.nodes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_program;

    fn tree(src: &str) -> ParseTree {
        parse_tree(&parse_program(src).unwrap(), src)
    }

    fn compact(tree: &ParseTree) -> String {
        match tree {
            ParseTree::Token { text, .. } => text.clone(),
            ParseTree::Rule { rule, children, .. } => {
                let children: Vec<String> = children.iter().map(compact).collect();
                format!("{rule}({})", children.join(" "))
            }
        }
    }

    fn count(tree: &ParseTree) -> (usize, usize) {
        match tree {
            ParseTree::Token { .. } => (0, 1),
            ParseTree::Rule { children, .. } => children
                .iter()
                .map(count)
                .fold((1, 0), |(r, l), (cr, cl)| (r + cr, l + cl)),
        }
    }

    #[test]
    fn leaves_of_practicum_example() {
        let src = "begin\n  R := 1;\n  while Y > 0 do\n  begin\n    R := R * X;\n    Y := Y - 1\n  end\nend";
        assert_eq!(
            tree(src).leaves(),
            [
                "begin", "R", ":=", "1", ";", "while", "Y", ">", "0", "do", "begin", "R", ":=",
                "R", "*", "X", ";", "Y", ":=", "Y", "-", "1", "end", "end",
            ]
        );
    }

    #[test]
    fn parentheses_and_negative_numbers() {
        assert_eq!(
            compact(&tree("begin X := (1 + 2) * -3 end")),
            "NP1(begin NS1(NV(X) := NA5(NA6(( NA3(NA1(NN(1)) + NA1(NN(2))) )) * NA1(NN(-3)))) end)"
        );
        assert_eq!(
            compact(&tree("begin X := ((Y)) end")),
            "NP1(begin NS1(NV(X) := NA6(( NA6(( NA2(NV(Y)) )) ))) end)"
        );
    }

    #[test]
    fn conditions() {
        assert_eq!(
            compact(&tree("begin if ¬(X = 1) ∨ (Y) > 2 then skip else skip end")),
            concat!(
                "NP1(begin NS3(if ",
                "NB3(NB4(¬ NB5(( NB1(NA2(NV(X)) = NA1(NN(1))) ))) ∨ NB2(NA6(( NA2(NV(Y)) )) > NA1(NN(2)))) ",
                "then NS6(skip) else NS6(skip)) end)"
            )
        );
    }

    #[test]
    fn statements() {
        assert_eq!(
            compact(&tree("begin skip; begin while X > 0 do X := X - 1 end end")),
            concat!(
                "NP1(begin NS2(NS6(skip) ; NS5(begin ",
                "NS4(while NB2(NA2(NV(X)) > NA1(NN(0))) do NS1(NV(X) := NA4(NA2(NV(X)) - NA1(NN(1))))) ",
                "end)) end)"
            )
        );
    }

    #[test]
    fn text_shows_rules_positions_and_leaves() {
        assert_eq!(
            to_text(&tree("begin\n  skip\nend"), "begin\n  skip\nend"),
            concat!(
                "<програма> NP1  1:1  begin…\n",
                "├── begin\n",
                "├── <оператор> NS6  2:3  skip\n",
                "│   └── skip\n",
                "└── end\n",
                "\n",
                "Листки зліва направо: 1 begin 2 skip 3 end\n",
            )
        );
    }

    #[test]
    fn json_links_nodes_to_source() {
        let src = "begin skip end";
        let json = to_json(&tree(src), src);
        assert_eq!(json["symbol"], "<програма>");
        assert_eq!(json["rule"], "NP1");
        assert_eq!(json["children"][0]["token"], "begin");
        assert_eq!(json["children"][1]["rule"], "NS6");
        assert_eq!(json["children"][1]["span"]["start"]["column"], 7);
        assert_eq!(json["children"][1]["span"]["end"]["column"], 11);
    }

    #[test]
    fn dot() {
        let src = "begin skip end";
        assert_eq!(
            to_dot(&tree(src), src),
            concat!(
                "digraph parse_tree {\n",
                "  ordering=out;\n",
                "  node [fontname=\"Helvetica\"];\n",
                "  n0 [label=\"<програма>\\nNP1\", shape=box, style=rounded, tooltip=\"1:1  begin skip end\"];\n",
                "  n1 [label=\"begin\", shape=plaintext, tooltip=\"1:1  begin\"];\n",
                "  n0 -> n1;\n",
                "  n2 [label=\"<оператор>\\nNS6\", shape=box, style=rounded, tooltip=\"1:7  skip\"];\n",
                "  n3 [label=\"skip\", shape=plaintext, tooltip=\"1:7  skip\"];\n",
                "  n2 -> n3;\n",
                "  n0 -> n2;\n",
                "  n4 [label=\"end\", shape=plaintext, tooltip=\"1:12  end\"];\n",
                "  n0 -> n4;\n",
                "}\n",
            )
        );
        assert_eq!(escape_dot(r#"a"b\c"#), r#"a\"b\\c"#);
    }

    #[test]
    fn svg_has_a_group_per_node_and_numbered_leaves() {
        let src = "begin\n  R := 1;\n  while Y > 0 do Y := Y - 1\nend";
        let tree = tree(src);
        let svg = to_svg(&tree, src);
        let (rules, leaves) = count(&tree);
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.ends_with("</svg>\n"));
        assert_eq!(svg.matches("<g class=\"rule\">").count(), rules);
        assert_eq!(svg.matches("<g class=\"leaf\">").count(), leaves);
        assert_eq!(svg.matches("<line ").count(), rules + leaves - 1);
        assert!(svg.contains(&format!(">{leaves}</text></g>")));
        assert!(svg.contains("<title>3:3  while Y &gt; 0 do Y := Y - 1</title>"));
        assert!(svg.contains(">&lt;умова&gt;</text>"));
    }
}
