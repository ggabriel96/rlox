use crate::expr::Expr;
use crate::lexing::{LiteralValue, Loc, Token, TokenKind};

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
    loc: Loc,
}

pub fn execute(expr: Expr) -> Result<Option<LiteralValue>, RuntimeError> {
    match expr {
        Expr::Binary { left, op, right } => {
            let left = execute(*left)?;
            let right = execute(*right)?;
            match op.kind {
                TokenKind::BangEqual => Ok(Some(LiteralValue::Bool(!is_equal(left, right)))),
                TokenKind::EqualEqual => Ok(Some(LiteralValue::Bool(is_equal(left, right)))),
                TokenKind::Greater => {
                    let (lhs, rhs) = expect_numbers(left, op, right)?;
                    Ok(Some(LiteralValue::Bool(lhs > rhs)))
                }
                TokenKind::GreaterEqual => {
                    let (lhs, rhs) = expect_numbers(left, op, right)?;
                    Ok(Some(LiteralValue::Bool(lhs >= rhs)))
                }
                TokenKind::Less => {
                    let (lhs, rhs) = expect_numbers(left, op, right)?;
                    Ok(Some(LiteralValue::Bool(lhs < rhs)))
                }
                TokenKind::LessEqual => {
                    let (lhs, rhs) = expect_numbers(left, op, right)?;
                    Ok(Some(LiteralValue::Bool(lhs <= rhs)))
                }
                TokenKind::Minus => {
                    let (lhs, rhs) = expect_numbers(left, op, right)?;
                    Ok(Some(LiteralValue::Number(lhs - rhs)))
                }
                TokenKind::Plus => match (left, right) {
                    (Some(LiteralValue::Number(lhs)), Some(LiteralValue::Number(rhs))) => {
                        Ok(Some(LiteralValue::Number(lhs + rhs)))
                    }
                    (Some(LiteralValue::String(lhs)), Some(LiteralValue::String(rhs))) => {
                        Ok(Some(LiteralValue::String(lhs + &rhs)))
                    }
                    (_, _) => Err(RuntimeError {
                        message: format!(
                            "Operator {} expects either two numeric or two string operands",
                            op.lexeme
                        ),
                        loc: op.loc,
                    }),
                },
                TokenKind::Slash => {
                    let (lhs, rhs) = expect_numbers(left, op, right)?;
                    Ok(Some(LiteralValue::Number(lhs / rhs)))
                }
                TokenKind::Star => {
                    let (lhs, rhs) = expect_numbers(left, op, right)?;
                    Ok(Some(LiteralValue::Number(lhs * rhs)))
                }
                _ => Err(RuntimeError {
                    message: format!("Invalid binary operator {}", op.lexeme),
                    loc: op.loc,
                }),
            }
        }
        Expr::Grouping { expr } => execute(*expr),
        Expr::Literal { value } => Ok(value),
        Expr::Unary { op, right } => {
            let right = execute(*right)?;
            match op {
                Token {
                    kind: TokenKind::Minus,
                    ..
                } => {
                    let rhs = expect_number(op, right)?;
                    Ok(Some(LiteralValue::Number(-rhs)))
                }
                Token {
                    kind: TokenKind::Bang,
                    ..
                } => Ok(Some(LiteralValue::Bool(!is_truthy(right)))),
                tok => Err(RuntimeError {
                    message: String::from("invalid unary operator?"),
                    loc: tok.loc,
                }),
            }
        }
    }
}

pub fn stringify(value: Option<LiteralValue>) -> String {
    match value {
        None => String::from("nil"),
        Some(LiteralValue::Bool(b)) => b.to_string(),
        Some(LiteralValue::Number(n)) => n.to_string(),
        Some(LiteralValue::String(s)) => s,
    }
}

fn expect_number(op: Token, rhs: Option<LiteralValue>) -> Result<f64, RuntimeError> {
    match rhs {
        Some(LiteralValue::Number(rhs)) => Ok(rhs),
        _ => Err(RuntimeError {
            message: format!("Unary operator {} expects a numeric operand", op.lexeme),
            loc: op.loc,
        }),
    }
}

fn expect_numbers(
    lhs: Option<LiteralValue>,
    op: Token,
    rhs: Option<LiteralValue>,
) -> Result<(f64, f64), RuntimeError> {
    match (lhs, rhs) {
        (Some(LiteralValue::Number(lhs)), Some(LiteralValue::Number(rhs))) => Ok((lhs, rhs)),
        (_, _) => Err(RuntimeError {
            message: format!("Binary operator {} expects two numeric operands", op.lexeme),
            loc: op.loc,
        }),
    }
}

fn is_equal(lhs: Option<LiteralValue>, rhs: Option<LiteralValue>) -> bool {
    match (lhs, rhs) {
        (None, None) => true,
        (None, _) | (_, None) => false,
        (Some(lhs), Some(rhs)) => lhs == rhs,
    }
}

fn is_truthy(expr: Option<LiteralValue>) -> bool {
    match expr {
        Some(LiteralValue::Bool(boolean)) => boolean,
        None => false,
        _ => true,
    }
}
