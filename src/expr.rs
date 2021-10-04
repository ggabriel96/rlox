use crate::lexing::{LiteralValue, Token};

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Option<LiteralValue>,
    },
    Unary {
        op: Token,
        right: Box<Expr>,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        let string = match self {
            Expr::Binary { left, op, right } => parenthesize(&op.lexeme, &[left, right]),
            Expr::Grouping { expr } => parenthesize(&"group", &[expr]),
            Expr::Literal { value: None } => String::from("nil"),
            Expr::Literal {
                value: Some(LiteralValue::Bool(b)),
            } => b.to_string(),
            Expr::Literal {
                value: Some(LiteralValue::Number(n)),
            } => n.to_string(),
            Expr::Literal {
                value: Some(LiteralValue::String(s)),
            } => s.clone(),
            Expr::Unary { op, right } => parenthesize(&op.lexeme, &[right.as_ref()]),
        };
        write!(f, "{}", string)
    }
}

fn parenthesize(name: &str, exprs: &[&Expr]) -> String {
    let mut buf = vec![String::from("("), String::from(name)];
    for expr in exprs.iter() {
        buf.push(String::from(" "));
        buf.push(expr.to_string());
    }
    buf.push(String::from(")"));
    buf.concat()
}
