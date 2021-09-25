use crate::expr::Expr;
use crate::lexing::{Token, TokenKind};
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
struct ParseError {
    message: String,
    token: Token,
}

pub fn parse(tokens: &Vec<Token>) -> Option<Expr> {
    if !tokens.is_empty() {
        let mut it = tokens.iter().peekable();
        match expression(&mut it) {
            Ok(expr) => Some(expr),
            Err(error) => panic!("{:?}", error),
        }
    } else {
        None
    }
}

fn expression(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParseError> {
    equality(it)
}

fn equality(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParseError> {
    let mut left = comparison(it);
    while let Some(Token {
        kind: TokenKind::BangEqual | TokenKind::EqualEqual,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = comparison(it);
        left = Ok(Expr::Binary {
            left: Box::new(left?),
            op: op.clone(),
            right: Box::new(right?),
        });
    }
    left
}

fn comparison(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParseError> {
    let mut left = term(it);
    while let Some(Token {
        kind: TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::Less | TokenKind::LessEqual,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = term(it);
        left = Ok(Expr::Binary {
            left: Box::new(left?),
            op: op.clone(),
            right: Box::new(right?),
        });
    }
    left
}

fn term(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParseError> {
    let mut left = factor(it);
    while let Some(Token {
        kind: TokenKind::Minus | TokenKind::Plus,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = factor(it);
        left = Ok(Expr::Binary {
            left: Box::new(left?),
            op: op.clone(),
            right: Box::new(right?),
        });
    }
    left
}

fn factor(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParseError> {
    let mut left = unary(it);
    while let Some(Token {
        kind: TokenKind::Slash | TokenKind::Star,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = unary(it);
        left = Ok(Expr::Binary {
            left: Box::new(left?),
            op: op.clone(),
            right: Box::new(right?),
        });
    }
    left
}

fn unary(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParseError> {
    if let Some(Token {
        kind: TokenKind::Bang | TokenKind::Minus,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = unary(it);
        Ok(Expr::Unary {
            op: op.clone(),
            right: Box::new(right?),
        })
    } else {
        primary(it)
    }
}

fn primary(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParseError> {
    match it.next() {
        Some(Token {
            kind:
                TokenKind::False
                | TokenKind::True
                | TokenKind::Nil
                | TokenKind::Number
                | TokenKind::String,
            literal,
            ..
        }) => Ok(Expr::Literal {
            value: literal.clone(),
        }),
        Some(open_paren) if matches!(open_paren.kind, TokenKind::LeftParen) => {
            let expr = expression(it);
            expect_closing_paren(open_paren, it)?;
            Ok(Expr::Grouping {
                expr: Box::new(expr?),
            })
        }
        Some(token) => Err(ParseError {
            message: String::from("Unexpected token"),
            token: token.clone(),
        }),
        None => panic!("Unexpected EOF"),
    }
}

fn expect_closing_paren(
    open_paren: &Token,
    it: &mut Peekable<Iter<Token>>,
) -> Result<(), ParseError> {
    match it.next() {
        Some(Token {
            kind: TokenKind::RightParen,
            ..
        }) => Ok(()),
        Some(not_close_paren) => Err(ParseError {
            message: String::from("Expected ')'"),
            token: not_close_paren.clone(),
        }),
        None => Err(ParseError {
            message: String::from("Expected ')', got EOF"),
            token: open_paren.clone(),
        }),
    }
}
