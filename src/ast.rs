use std::fmt;

use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub body: Statement,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementKind {
    Assign {
        variable: Variable,
        value: ArithExpr,
    },
    Sequence(Box<Statement>, Box<Statement>),
    If {
        condition: BoolExpr,
        then_branch: Box<Statement>,
        else_branch: Box<Statement>,
    },
    While {
        condition: BoolExpr,
        body: Box<Statement>,
    },
    Block(Box<Statement>),
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArithExpr {
    pub kind: ArithExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithExprKind {
    Number(i64),
    Variable(String),
    Binary {
        op: ArithOp,
        left: Box<ArithExpr>,
        right: Box<ArithExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    Add,
    Subtract,
    Multiply,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoolExpr {
    pub kind: BoolExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoolExprKind {
    Compare {
        op: ComparisonOp,
        left: ArithExpr,
        right: ArithExpr,
    },
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    Equal,
    Greater,
}

impl ArithOp {
    pub fn symbol(self) -> &'static str {
        match self {
            ArithOp::Add => "+",
            ArithOp::Subtract => "-",
            ArithOp::Multiply => "*",
        }
    }
}

impl ComparisonOp {
    pub fn symbol(self) -> &'static str {
        match self {
            ComparisonOp::Equal => "=",
            ComparisonOp::Greater => ">",
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.body)
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            StatementKind::Assign { variable, value } => {
                write!(f, "(:= {} {})", variable.name, value)
            }
            StatementKind::Sequence(a, b) => write!(f, "(; {a} {b})"),
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                write!(f, "(if {condition} {then_branch} {else_branch})")
            }
            StatementKind::While { condition, body } => write!(f, "(while {condition} {body})"),
            StatementKind::Block(s) => write!(f, "(begin {s})"),
            StatementKind::Skip => write!(f, "skip"),
        }
    }
}

impl fmt::Display for ArithExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ArithExprKind::Number(n) => write!(f, "{n}"),
            ArithExprKind::Variable(name) => write!(f, "{name}"),
            ArithExprKind::Binary { op, left, right } => {
                write!(f, "({} {left} {right})", op.symbol())
            }
        }
    }
}

impl fmt::Display for BoolExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            BoolExprKind::Compare { op, left, right } => {
                write!(f, "({} {left} {right})", op.symbol())
            }
            BoolExprKind::Or(a, b) => write!(f, "(or {a} {b})"),
            BoolExprKind::Not(b) => write!(f, "(not {b})"),
        }
    }
}
