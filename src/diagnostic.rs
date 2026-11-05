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

#[cfg(test)]
mod tests {
    use super::*;

    fn render(src: &str, start: usize, end: usize) -> String {
        Diagnostic::new("повідомлення", Span::new(start, end)).render(src, "f.sipl")
    }

    #[test]
    fn underlines_span_on_its_line() {
        assert_eq!(
            render("begin\n  X := 1;\nend", 14, 15),
            "помилка: повідомлення\n --> f.sipl:2:9\n  |\n2 |   X := 1;\n  |         ^\n"
        );
    }

    #[test]
    fn columns_count_characters_not_bytes() {
        assert_eq!(
            render("¬ ¬ X", 6, 7),
            "помилка: повідомлення\n --> f.sipl:1:5\n  |\n1 | ¬ ¬ X\n  |     ^\n"
        );
    }

    #[test]
    fn multiline_span_is_cut_at_end_of_line() {
        assert_eq!(
            render("while X > 0 do\n  skip", 0, 21),
            "помилка: повідомлення\n --> f.sipl:1:1\n  |\n1 | while X > 0 do\n  | ^^^^^^^^^^^^^^\n"
        );
    }

    #[test]
    fn end_of_file_gets_one_caret() {
        assert_eq!(
            render("begin", 5, 5),
            "помилка: повідомлення\n --> f.sipl:1:6\n  |\n1 | begin\n  |      ^\n"
        );
    }

    #[test]
    fn gutter_widens_for_long_line_numbers() {
        let src = format!("{}X", "\n".repeat(9));
        assert_eq!(
            render(&src, 9, 10),
            "помилка: повідомлення\n  --> f.sipl:10:1\n   |\n10 | X\n   | ^\n"
        );
    }
}
