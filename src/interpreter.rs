use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::state::State;

pub const DEFAULT_ITERATION_LIMIT: u64 = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    UndefinedVariable(String),
    Overflow { op: ArithOp, left: i64, right: i64 },
    IterationLimitExceeded(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub span: Span,
}

impl RuntimeError {
    pub fn message(&self) -> String {
        match &self.kind {
            RuntimeErrorKind::UndefinedVariable(name) => {
                format!("змінна `{name}` не має значення в поточному стані")
            }
            RuntimeErrorKind::Overflow { op, left, right } => {
                format!(
                    "переповнення: {left} {} {right} не вміщується в 64-бітне ціле",
                    op.symbol()
                )
            }
            RuntimeErrorKind::IterationLimitExceeded(limit) => {
                format!("перевищено ліміт у {limit} ітерацій циклів, можливо, цикл не завершується")
            }
        }
    }

    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::new(self.message(), self.span)
    }
}

pub fn run(program: &Program, state: State) -> Result<State, RuntimeError> {
    run_with_iteration_limit(program, state, DEFAULT_ITERATION_LIMIT)
}

pub fn run_with_iteration_limit(
    program: &Program,
    state: State,
    iteration_limit: u64,
) -> Result<State, RuntimeError> {
    let mut interpreter = Interpreter {
        state,
        iterations: 0,
        iteration_limit,
    };
    interpreter.execute(&program.body)?;
    Ok(interpreter.state)
}

struct Interpreter {
    state: State,
    iterations: u64,
    iteration_limit: u64,
}

impl Interpreter {
    fn count_iteration(&mut self, loop_condition: Span) -> Result<(), RuntimeError> {
        self.iterations += 1;
        if self.iterations > self.iteration_limit {
            return Err(RuntimeError {
                kind: RuntimeErrorKind::IterationLimitExceeded(self.iteration_limit),
                span: loop_condition,
            });
        }
        Ok(())
    }

    fn execute(&mut self, statement: &Statement) -> Result<(), RuntimeError> {
        match &statement.kind {
            StatementKind::Assign { variable, value } => {
                let value = self.evaluate(value)?;
                self.state.set(&variable.name, value);
            }
            StatementKind::Sequence(first, second) => {
                self.execute(first)?;
                self.execute(second)?;
            }
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.check(condition)? {
                    self.execute(then_branch)?;
                } else {
                    self.execute(else_branch)?;
                }
            }
            StatementKind::While { condition, body } => {
                while self.check(condition)? {
                    self.count_iteration(condition.span)?;
                    self.execute(body)?;
                }
            }
            StatementKind::Block(inner) => self.execute(inner)?,
            StatementKind::Skip => {}
        }
        Ok(())
    }

    fn evaluate(&self, expr: &ArithExpr) -> Result<i64, RuntimeError> {
        match &expr.kind {
            ArithExprKind::Number(n) => Ok(*n),
            ArithExprKind::Variable(name) => self.state.get(name).ok_or_else(|| RuntimeError {
                kind: RuntimeErrorKind::UndefinedVariable(name.clone()),
                span: expr.span,
            }),
            ArithExprKind::Binary { op, left, right } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                let result = match op {
                    ArithOp::Add => left.checked_add(right),
                    ArithOp::Subtract => left.checked_sub(right),
                    ArithOp::Multiply => left.checked_mul(right),
                };
                result.ok_or(RuntimeError {
                    kind: RuntimeErrorKind::Overflow {
                        op: *op,
                        left,
                        right,
                    },
                    span: expr.span,
                })
            }
        }
    }

    fn check(&self, condition: &BoolExpr) -> Result<bool, RuntimeError> {
        match &condition.kind {
            BoolExprKind::Compare { op, left, right } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                Ok(match op {
                    ComparisonOp::Equal => left == right,
                    ComparisonOp::Greater => left > right,
                })
            }
            BoolExprKind::Or(first, second) => {
                let first = self.check(first)?;
                let second = self.check(second)?;
                Ok(first || second)
            }
            BoolExprKind::Not(inner) => Ok(!self.check(inner)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_program;

    fn run_source(src: &str, state: State) -> Result<State, RuntimeError> {
        run(&parse_program(src).unwrap(), state)
    }

    fn final_state(src: &str, state: State) -> String {
        run_source(src, state).unwrap().to_string()
    }

    fn error_at(src: &str, state: State) -> (RuntimeErrorKind, &str) {
        let err = run_source(src, state).unwrap_err();
        (err.kind, &src[err.span.start..err.span.end])
    }

    #[test]
    fn power_from_practicum() {
        let src = "begin R := 1; while Y > 0 do begin R := R * X; Y := Y - 1 end end";
        assert_eq!(
            final_state(src, State::from([("X", 2), ("Y", 10)])),
            "[R ↦ 1024, X ↦ 2, Y ↦ 0]"
        );
    }

    #[test]
    fn if_takes_only_one_branch() {
        let src = "begin if X > 0 then Y := 1 else Y := Z end";
        assert_eq!(final_state(src, State::from([("X", 5)])), "[X ↦ 5, Y ↦ 1]");
    }

    #[test]
    fn while_with_false_condition_does_not_run_body() {
        let src = "begin while X > 0 do Y := Z end";
        assert_eq!(final_state(src, State::from([("X", 0)])), "[X ↦ 0]");
    }

    #[test]
    fn skip_and_blocks_keep_state() {
        let src = "begin begin skip end; skip end";
        assert_eq!(final_state(src, State::from([("A", 1)])), "[A ↦ 1]");
    }

    #[test]
    fn undefined_variable_points_to_usage() {
        let src = "begin X := Y + 1 end";
        assert_eq!(
            error_at(src, State::new()),
            (RuntimeErrorKind::UndefinedVariable("Y".into()), "Y")
        );
    }

    #[test]
    fn or_is_strict_like_superposition() {
        let src = "begin if X = 0 or Y > 0 then skip else skip end";
        assert_eq!(
            error_at(src, State::from([("X", 0)])),
            (RuntimeErrorKind::UndefinedVariable("Y".into()), "Y")
        );
    }

    #[test]
    fn overflow_is_reported() {
        let src = "begin X := 9223372036854775807; X := X * 2 end";
        assert_eq!(
            error_at(src, State::new()),
            (
                RuntimeErrorKind::Overflow {
                    op: ArithOp::Multiply,
                    left: i64::MAX,
                    right: 2
                },
                "X * 2"
            )
        );
    }

    #[test]
    fn infinite_loop_points_to_its_condition() {
        let src =
            "begin X := 0; while X > 0 do skip; while 1 > 0 do if X = 0 then skip else skip end";
        let program = parse_program(src).unwrap();
        let err = run_with_iteration_limit(&program, State::new(), 100).unwrap_err();
        assert_eq!(err.kind, RuntimeErrorKind::IterationLimitExceeded(100));
        assert_eq!(&src[err.span.start..err.span.end], "1 > 0");
    }

    #[test]
    fn iteration_limit_counts_body_executions_of_all_loops() {
        let program = parse_program(
            "begin X := 2; while X > 0 do X := X - 1; Y := 1; while Y > 0 do Y := 0 end",
        )
        .unwrap();
        assert!(run_with_iteration_limit(&program, State::new(), 3).is_ok());
        assert!(run_with_iteration_limit(&program, State::new(), 2).is_err());
    }

    #[test]
    fn error_messages() {
        let messages: Vec<String> = [
            RuntimeErrorKind::UndefinedVariable("X".into()),
            RuntimeErrorKind::Overflow {
                op: ArithOp::Add,
                left: 1,
                right: 2,
            },
            RuntimeErrorKind::IterationLimitExceeded(10),
        ]
        .into_iter()
        .map(|kind| {
            RuntimeError {
                kind,
                span: Span::new(0, 0),
            }
            .to_diagnostic()
            .message
        })
        .collect();
        assert_eq!(
            messages,
            [
                "змінна `X` не має значення в поточному стані",
                "переповнення: 1 + 2 не вміщується в 64-бітне ціле",
                "перевищено ліміт у 10 ітерацій циклів, можливо, цикл не завершується",
            ]
        );
    }
}
