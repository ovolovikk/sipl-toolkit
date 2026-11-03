use crate::span::{Span, line_col};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, span: Span) -> Diagnostic {
        Diagnostic {
            message: message.into(),
            span,
        }
    }

    pub fn short(&self, src: &str) -> String {
        let lc = line_col(src, self.span.start);
        format!("{}:{}: {}", lc.line, lc.col, self.message)
    }

    pub fn render(&self, src: &str, path: &str) -> String {
        let lc = line_col(src, self.span.start);
        let line_text = src
            .split('\n')
            .nth(lc.line - 1)
            .unwrap_or("")
            .trim_end_matches('\r');
        let width = src[self.span.start..self.span.end]
            .chars()
            .take_while(|&c| c != '\n')
            .count()
            .max(1);
        let num = lc.line.to_string();
        let pad = " ".repeat(num.len());
        format!(
            "помилка: {}\n{pad}--> {path}:{}:{}\n{pad} |\n{num} | {line_text}\n{pad} | {}{}\n",
            self.message,
            lc.line,
            lc.col,
            " ".repeat(lc.col - 1),
            "^".repeat(width),
        )
    }
}
