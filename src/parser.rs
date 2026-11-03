use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind, tokenize};
use crate::span::Span;

pub fn parse_program(src: &str) -> Result<Program, Diagnostic> {
    let tokens = tokenize(src)?;
    let mut parser = Parser { tokens, pos: 0 };
    parser.program()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn unexpected(&self, expected: &str) -> Diagnostic {
        let token = &self.tokens[self.pos];
        Diagnostic::new(
            format!("очікувалось {expected}, знайдено {}", token.kind.describe()),
            token.span,
        )
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, Diagnostic> {
        if *self.peek() == kind {
            Ok(self.bump())
        } else {
            Err(self.unexpected(&format!("`{}`", kind.text())))
        }
    }

    fn program(&mut self) -> Result<Program, Diagnostic> {
        let begin = self.expect(TokenKind::Begin)?;
        let body = self.statement()?;
        let end = self.expect(TokenKind::End)?;
        if *self.peek() != TokenKind::Eof {
            let token = &self.tokens[self.pos];
            return Err(Diagnostic::new(
                format!(
                    "після `end` програми нічого не має бути, знайдено {}",
                    token.kind.describe()
                ),
                token.span,
            ));
        }
        Ok(Program {
            body,
            span: begin.span.to(end.span),
        })
    }

    fn statement(&mut self) -> Result<Statement, Diagnostic> {
        let mut left = self.simple()?;
        while *self.peek() == TokenKind::Semicolon {
            let semi = self.bump();
            if *self.peek() == TokenKind::End {
                return Err(Diagnostic::new(
                    "зайва `;` перед `end`: у SIPL `;` розділяє оператори, а не завершує їх",
                    semi.span,
                ));
            }
            let right = self.simple()?;
            let span = left.span.to(right.span);
            left = Statement {
                kind: StatementKind::Sequence(Box::new(left), Box::new(right)),
                span,
            };
        }
        if starts_statement(self.peek()) {
            let token = &self.tokens[self.pos];
            return Err(Diagnostic::new(
                format!(
                    "пропущено `;` між операторами, знайдено {}",
                    token.kind.describe()
                ),
                token.span,
            ));
        }
        Ok(left)
    }

    fn simple(&mut self) -> Result<Statement, Diagnostic> {
        match self.peek().clone() {
            TokenKind::Identifier(name) => {
                let token = self.bump();
                self.expect(TokenKind::Assign)?;
                let value = self.expression()?;
                let span = token.span.to(value.span);
                Ok(Statement {
                    kind: StatementKind::Assign {
                        variable: Variable {
                            name,
                            span: token.span,
                        },
                        value,
                    },
                    span,
                })
            }
            TokenKind::If => {
                let token = self.bump();
                let condition = self.condition()?;
                self.expect(TokenKind::Then)?;
                let then_branch = self.simple()?;
                self.expect(TokenKind::Else)?;
                let else_branch = self.simple()?;
                let span = token.span.to(else_branch.span);
                let kind = StatementKind::If {
                    condition,
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                };
                Ok(Statement { kind, span })
            }
            TokenKind::While => {
                let token = self.bump();
                let condition = self.condition()?;
                self.expect(TokenKind::Do)?;
                let body = self.simple()?;
                let span = token.span.to(body.span);
                Ok(Statement {
                    kind: StatementKind::While {
                        condition,
                        body: Box::new(body),
                    },
                    span,
                })
            }
            TokenKind::Begin => {
                let begin = self.bump();
                let inner = self.statement()?;
                let end = self.expect(TokenKind::End)?;
                Ok(Statement {
                    kind: StatementKind::Block(Box::new(inner)),
                    span: begin.span.to(end.span),
                })
            }
            TokenKind::Skip => {
                let token = self.bump();
                Ok(Statement {
                    kind: StatementKind::Skip,
                    span: token.span,
                })
            }
            TokenKind::End
                if self.pos > 0 && self.tokens[self.pos - 1].kind == TokenKind::Begin =>
            {
                Err(self.unexpected("оператор (порожній блок записується як `begin skip end`)"))
            }
            _ => Err(self.unexpected("оператор")),
        }
    }

    fn expression(&mut self) -> Result<ArithExpr, Diagnostic> {
        let mut left = self.term()?;
        loop {
            let op = match self.peek() {
                TokenKind::Plus => ArithOp::Add,
                TokenKind::Minus => ArithOp::Subtract,
                _ => return Ok(left),
            };
            self.bump();
            let right = self.term()?;
            left = binary(op, left, right);
        }
    }

    fn term(&mut self) -> Result<ArithExpr, Diagnostic> {
        let mut left = self.factor()?;
        while *self.peek() == TokenKind::Star {
            self.bump();
            let right = self.factor()?;
            left = binary(ArithOp::Multiply, left, right);
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<ArithExpr, Diagnostic> {
        match self.peek().clone() {
            TokenKind::Number(digits) => {
                let token = self.bump();
                number(&digits, token.span)
            }
            TokenKind::Minus => {
                let minus = self.bump();
                match self.peek().clone() {
                    TokenKind::Number(digits) => {
                        let token = self.bump();
                        number(&format!("-{digits}"), minus.span.to(token.span))
                    }
                    _ => Err(self
                        .unexpected("число після `-` (унарного мінуса для змінних у SIPL немає)")),
                }
            }
            TokenKind::Identifier(name) => {
                let token = self.bump();
                Ok(ArithExpr {
                    kind: ArithExprKind::Variable(name),
                    span: token.span,
                })
            }
            TokenKind::LeftParen => {
                let open = self.bump();
                let inner = self.expression()?;
                let close = self.expect(TokenKind::RightParen)?;
                Ok(ArithExpr {
                    kind: inner.kind,
                    span: open.span.to(close.span),
                })
            }
            _ => Err(self.unexpected("арифметичний вираз")),
        }
    }

    fn condition(&mut self) -> Result<BoolExpr, Diagnostic> {
        let mut left = self.disjunct()?;
        while *self.peek() == TokenKind::Or {
            self.bump();
            let right = self.disjunct()?;
            let span = left.span.to(right.span);
            left = BoolExpr {
                kind: BoolExprKind::Or(Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn disjunct(&mut self) -> Result<BoolExpr, Diagnostic> {
        if *self.peek() == TokenKind::Not {
            let not = self.bump();
            let inner = self.disjunct()?;
            let span = not.span.to(inner.span);
            return Ok(BoolExpr {
                kind: BoolExprKind::Not(Box::new(inner)),
                span,
            });
        }

        if *self.peek() == TokenKind::LeftParen && self.paren_holds_condition() {
            let open = self.bump();
            let inner = self.condition()?;
            let close = self.expect(TokenKind::RightParen)?;
            return Ok(BoolExpr {
                kind: inner.kind,
                span: open.span.to(close.span),
            });
        }

        let left = self.expression()?;
        let op = match self.peek() {
            TokenKind::Equal => ComparisonOp::Equal,
            TokenKind::Greater => ComparisonOp::Greater,
            _ => return Err(self.unexpected("`=` або `>`")),
        };
        self.bump();
        let right = self.expression()?;
        let span = left.span.to(right.span);
        Ok(BoolExpr {
            kind: BoolExprKind::Compare { op, left, right },
            span,
        })
    }

    fn paren_holds_condition(&self) -> bool {
        let mut depth = 0;
        for i in self.pos..self.tokens.len() {
            match self.tokens[i].kind {
                TokenKind::LeftParen => depth += 1,
                TokenKind::RightParen => {
                    depth -= 1;
                    if depth == 0 {
                        return !matches!(
                            self.tokens[i + 1].kind,
                            TokenKind::Equal
                                | TokenKind::Greater
                                | TokenKind::Plus
                                | TokenKind::Minus
                                | TokenKind::Star
                        );
                    }
                }
                _ => {}
            }
        }
        true
    }
}

fn starts_statement(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Identifier(_)
            | TokenKind::If
            | TokenKind::While
            | TokenKind::Begin
            | TokenKind::Skip
    )
}

fn binary(op: ArithOp, left: ArithExpr, right: ArithExpr) -> ArithExpr {
    let span = left.span.to(right.span);
    ArithExpr {
        kind: ArithExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
        span,
    }
}

fn number(text: &str, span: Span) -> Result<ArithExpr, Diagnostic> {
    match text.parse::<i64>() {
        Ok(n) => Ok(ArithExpr {
            kind: ArithExprKind::Number(n),
            span,
        }),
        Err(_) => Err(Diagnostic::new(
            format!("число `{text}` не вміщується в 64-бітне ціле"),
            span,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sexpr(src: &str) -> String {
        parse_program(src).unwrap().to_string()
    }

    fn condition(text: &str) -> String {
        let program = format!("begin if {text} then skip else skip end");
        match parse_program(&program).unwrap().body.kind {
            StatementKind::If { condition, .. } => condition.to_string(),
            _ => unreachable!(),
        }
    }

    #[test]
    fn arithmetic_precedence_and_associativity() {
        assert_eq!(
            sexpr("begin X := 1 + 2 * 3 - 4 end"),
            "(:= X (- (+ 1 (* 2 3)) 4))"
        );
        assert_eq!(sexpr("begin X := 10 - 3 - 2 end"), "(:= X (- (- 10 3) 2))");
        assert_eq!(sexpr("begin X := (1 + 2) * 3 end"), "(:= X (* (+ 1 2) 3))");
    }

    #[test]
    fn negative_literals() {
        assert_eq!(sexpr("begin X := -1 end"), "(:= X -1)");
        assert_eq!(sexpr("begin X := Y - -2 end"), "(:= X (- Y -2))");
        assert_eq!(sexpr("begin X := Y-1 end"), "(:= X (- Y 1))");
    }

    #[test]
    fn sequence_is_weaker_than_while() {
        assert_eq!(
            sexpr("begin while X > 0 do X := X - 1; Y := 1 end"),
            "(; (while (> X 0) (:= X (- X 1))) (:= Y 1))"
        );
        assert_eq!(
            sexpr("begin skip; skip; skip end"),
            "(; (; skip skip) skip)"
        );
    }

    #[test]
    fn conditions() {
        assert_eq!(
            condition("not X = 1 or Y > 2"),
            "(or (not (= X 1)) (> Y 2))"
        );
        assert_eq!(condition("¬(X = 1 ∨ Y > 2)"), "(not (or (= X 1) (> Y 2)))");
        assert_eq!(condition("(X + 1) > 2"), "(> (+ X 1) 2)");
        assert_eq!(condition("(X) = (Y)"), "(= X Y)");
        assert_eq!(condition("((X > 1))"), "(> X 1)");
        assert_eq!(
            condition("((X) * 2 > 1) or (1 = Y)"),
            "(or (> (* X 2) 1) (= 1 Y))"
        );
    }

    #[test]
    fn spans_cover_source() {
        let src = "begin\n  R := (1 + X) * 2\nend";
        let program = parse_program(src).unwrap();
        assert_eq!(&src[program.span.start..program.span.end], src);
        let StatementKind::Assign { variable, value } = &program.body.kind else {
            panic!()
        };
        assert_eq!(&src[variable.span.start..variable.span.end], "R");
        assert_eq!(&src[value.span.start..value.span.end], "(1 + X) * 2");
        let ArithExprKind::Binary { left, .. } = &value.kind else {
            panic!()
        };
        assert_eq!(&src[left.span.start..left.span.end], "(1 + X)");
    }

    #[test]
    fn diagnostics_point_to_line_and_column() {
        let src = "begin\n  X := 1;\nend";
        let err = parse_program(src).unwrap_err();
        assert_eq!(
            err.short(src),
            "2:9: зайва `;` перед `end`: у SIPL `;` розділяє оператори, а не завершує їх"
        );
    }
}
