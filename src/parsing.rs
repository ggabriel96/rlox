use crate::expr::Expr;
use crate::lexing::{Token, TokenKind};
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse(tokens: &Vec<Token>) -> Expr {
    let mut it = tokens.iter().peekable();
    expression(&mut it)
}

fn expression(it: &mut Peekable<Iter<Token>>) -> Expr {
    equality(it)
}

fn equality(it: &mut Peekable<Iter<Token>>) -> Expr {
    let mut left = comparison(it);
    while let Some(Token {
        kind: TokenKind::BangEqual | TokenKind::EqualEqual,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = comparison(it);
        left = Expr::Binary {
            left: Box::new(left),
            op: op.clone(),
            right: Box::new(right),
        };
    }
    left
}

fn comparison(it: &mut Peekable<Iter<Token>>) -> Expr {
    let mut left = term(it);
    while let Some(Token {
        kind: TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::Less | TokenKind::LessEqual,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = term(it);
        left = Expr::Binary {
            left: Box::new(left),
            op: op.clone(),
            right: Box::new(right),
        };
    }
    left
}

fn term(it: &mut Peekable<Iter<Token>>) -> Expr {
    let mut left = factor(it);
    while let Some(Token {
        kind: TokenKind::Minus | TokenKind::Plus,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = factor(it);
        left = Expr::Binary {
            left: Box::new(left),
            op: op.clone(),
            right: Box::new(right),
        };
    }
    left
}

fn factor(it: &mut Peekable<Iter<Token>>) -> Expr {
    let mut left = unary(it);
    while let Some(Token {
        kind: TokenKind::Slash | TokenKind::Star,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = unary(it);
        left = Expr::Binary {
            left: Box::new(left),
            op: op.clone(),
            right: Box::new(right),
        };
    }
    left
}

fn unary(it: &mut Peekable<Iter<Token>>) -> Expr {
    if let Some(Token {
        kind: TokenKind::Bang | TokenKind::Minus,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = unary(it);
        Expr::Unary {
            op: op.clone(),
            right: Box::new(right),
        }
    } else {
        primary(it)
    }
}

fn primary(it: &mut Peekable<Iter<Token>>) -> Expr {
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
        }) => Expr::Literal {
            value: literal.clone(),
        },
        Some(Token {
            kind: TokenKind::LeftParen,
            ..
        }) => {
            let expr = expression(it);
            match it.peek() {
                Some(Token {
                    kind: TokenKind::RightParen,
                    ..
                }) => (),
                Some(Token { loc, .. }) => {
                    panic!("Expected ')' after expression on line {}", loc.line_end)
                }
                None => panic!("Unexpected EOF"), // how to get line number here?
            }
            Expr::Grouping {
                expr: Box::new(expr),
            }
        }
        Some(tok) => panic!(
            "Unexpected token {} at line {}",
            tok.lexeme, tok.loc.line_begin
        ),
        None => panic!("Unexpected EOF"),
    }
}
