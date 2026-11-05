use crate::diagnostic::Diagnostic;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    Number(String),
    Begin,
    End,
    If,
    Then,
    Else,
    While,
    Do,
    Skip,
    Or,
    Not,
    Assign,
    Semicolon,
    Plus,
    Minus,
    Star,
    Equal,
    Greater,
    LeftParen,
    RightParen,
    Eof,
}

impl TokenKind {
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Identifier(name) => format!("змінну `{name}`"),
            TokenKind::Number(digits) => format!("число `{digits}`"),
            TokenKind::Eof => "кінець файлу".to_string(),
            other => format!("`{}`", other.text()),
        }
    }

    pub fn text(&self) -> &str {
        match self {
            TokenKind::Identifier(name) => name,
            TokenKind::Number(digits) => digits,
            TokenKind::Begin => "begin",
            TokenKind::End => "end",
            TokenKind::If => "if",
            TokenKind::Then => "then",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::Do => "do",
            TokenKind::Skip => "skip",
            TokenKind::Or => "or",
            TokenKind::Not => "not",
            TokenKind::Assign => ":=",
            TokenKind::Semicolon => ";",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Equal => "=",
            TokenKind::Greater => ">",
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::Eof => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

fn keyword(word: &str) -> Option<TokenKind> {
    Some(match word {
        "begin" => TokenKind::Begin,
        "end" => TokenKind::End,
        "if" => TokenKind::If,
        "then" => TokenKind::Then,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "do" => TokenKind::Do,
        "skip" => TokenKind::Skip,
        "or" => TokenKind::Or,
        "not" => TokenKind::Not,
        _ => return None,
    })
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, Diagnostic> {
    let chars: Vec<(usize, char)> = src.char_indices().collect();
    let offset = |i: usize| chars.get(i).map_or(src.len(), |&(pos, _)| pos);
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let (start, c) = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        if c.is_ascii_alphabetic() || c == '_' {
            let mut j = i;
            while j < chars.len() && (chars[j].1.is_ascii_alphanumeric() || chars[j].1 == '_') {
                j += 1;
            }
            let word = &src[start..offset(j)];
            let kind = keyword(word).unwrap_or_else(|| TokenKind::Identifier(word.to_string()));
            tokens.push(Token {
                kind,
                span: Span::new(start, offset(j)),
            });
            i = j;
            continue;
        }

        if c.is_ascii_digit() {
            let mut j = i;
            while j < chars.len() && chars[j].1.is_ascii_digit() {
                j += 1;
            }
            let digits = src[start..offset(j)].to_string();
            tokens.push(Token {
                kind: TokenKind::Number(digits),
                span: Span::new(start, offset(j)),
            });
            i = j;
            continue;
        }

        let one = Span::new(start, offset(i + 1));
        let kind = match c {
            ':' if chars.get(i + 1).map(|&(_, n)| n) == Some('=') => {
                tokens.push(Token {
                    kind: TokenKind::Assign,
                    span: Span::new(start, offset(i + 2)),
                });
                i += 2;
                continue;
            }
            ':' => {
                return Err(Diagnostic::new(
                    "очікувалось `:=`, знайдено одиночну `:`",
                    one,
                ));
            }
            ';' => TokenKind::Semicolon,
            '+' => TokenKind::Plus,
            '-' | '–' | '−' => TokenKind::Minus,
            '*' | '∗' | '×' => TokenKind::Star,
            '=' => TokenKind::Equal,
            '>' => TokenKind::Greater,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '∨' => TokenKind::Or,
            '¬' => TokenKind::Not,
            '<' => {
                return Err(Diagnostic::new(
                    "символу `<` немає в SIPL, є лише `=` і `>` (замість `a < b` пишіть `b > a`)",
                    one,
                ));
            }
            _ => return Err(Diagnostic::new(format!("невідомий символ `{c}`"), one)),
        };
        tokens.push(Token { kind, span: one });
        i += 1;
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(src.len(), src.len()),
    });
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn keywords_and_identifiers() {
        assert_eq!(
            kinds("begin R1 := beginx end"),
            vec![
                TokenKind::Begin,
                TokenKind::Identifier("R1".into()),
                TokenKind::Assign,
                TokenKind::Identifier("beginx".into()),
                TokenKind::End,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn unicode_operators_from_practicum() {
        assert_eq!(
            kinds("¬ ∨ – − ∗ ×"),
            vec![
                TokenKind::Not,
                TokenKind::Or,
                TokenKind::Minus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Star,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn spans_are_byte_offsets() {
        let tokens = tokenize("¬ X").unwrap();
        assert_eq!(tokens[0].span, Span::new(0, 2));
        assert_eq!(tokens[1].span, Span::new(3, 4));
        assert_eq!(tokens[2].span, Span::new(4, 4));
    }

    #[test]
    fn text_of_every_token_lexes_back_to_it() {
        let all = [
            TokenKind::Identifier("X".into()),
            TokenKind::Number("42".into()),
            TokenKind::Begin,
            TokenKind::End,
            TokenKind::If,
            TokenKind::Then,
            TokenKind::Else,
            TokenKind::While,
            TokenKind::Do,
            TokenKind::Skip,
            TokenKind::Or,
            TokenKind::Not,
            TokenKind::Assign,
            TokenKind::Semicolon,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Equal,
            TokenKind::Greater,
            TokenKind::LeftParen,
            TokenKind::RightParen,
        ];
        for kind in all {
            assert_eq!(kinds(kind.text()), vec![kind.clone(), TokenKind::Eof]);
        }
        assert_eq!(kinds(TokenKind::Eof.text()), vec![TokenKind::Eof]);
    }

    #[test]
    fn describe() {
        assert_eq!(TokenKind::Identifier("X".into()).describe(), "змінну `X`");
        assert_eq!(TokenKind::Number("42".into()).describe(), "число `42`");
        assert_eq!(TokenKind::While.describe(), "`while`");
        assert_eq!(TokenKind::Eof.describe(), "кінець файлу");
    }

    #[test]
    fn unknown_character() {
        let err = tokenize("X := 1 & 2").unwrap_err();
        assert_eq!(err.span, Span::new(7, 8));
    }
}
