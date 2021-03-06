use crate::expr::Expr;
use crate::lexing::{Token, TokenKind};
use crate::stmt::Stmt;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
pub struct ParsingError {
    message: String,
    token: Token,
}

pub fn parse(tokens: &Vec<Token>) -> Result<Vec<Stmt>, ParsingError> {
    let mut it = tokens.iter().peekable();
    let mut statements: Vec<Stmt> = vec![];
    loop {
        let stmt = match it.peek() {
            Some(Token {
                kind: TokenKind::Eof,
                ..
            }) => break,
            Some(Token {
                kind: TokenKind::Print,
                ..
            }) => {
                it.next(); // consume the peeked print token
                print_statement(&mut it)
            }
            _ => expression_statement(&mut it),
        };
        statements.push(stmt?);
    }
    Ok(statements)
}

fn expression_statement(it: &mut Peekable<Iter<Token>>) -> Result<Stmt, ParsingError> {
    let expr = expression(it)?;
    expect_semicolon(it)?;
    Ok(Stmt::Expr(expr))
}

fn print_statement(it: &mut Peekable<Iter<Token>>) -> Result<Stmt, ParsingError> {
    let expr = expression(it)?;
    expect_semicolon(it)?;
    Ok(Stmt::Print(expr))
}

fn expression(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParsingError> {
    equality(it)
}

fn equality(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParsingError> {
    let mut left = comparison(it)?;
    while let Some(Token {
        kind: TokenKind::BangEqual | TokenKind::EqualEqual,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = comparison(it)?;
        left = Expr::Binary {
            left: Box::new(left),
            op: op.clone(),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn comparison(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParsingError> {
    let mut left = term(it)?;
    while let Some(Token {
        kind: TokenKind::Greater | TokenKind::GreaterEqual | TokenKind::Less | TokenKind::LessEqual,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = term(it)?;
        left = Expr::Binary {
            left: Box::new(left),
            op: op.clone(),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn term(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParsingError> {
    let mut left = factor(it)?;
    while let Some(Token {
        kind: TokenKind::Minus | TokenKind::Plus,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = factor(it)?;
        left = Expr::Binary {
            left: Box::new(left),
            op: op.clone(),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn factor(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParsingError> {
    let mut left = unary(it)?;
    while let Some(Token {
        kind: TokenKind::Slash | TokenKind::Star,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = unary(it)?;
        left = Expr::Binary {
            left: Box::new(left),
            op: op.clone(),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn unary(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParsingError> {
    if let Some(Token {
        kind: TokenKind::Bang | TokenKind::Minus,
        ..
    }) = it.peek()
    {
        let op = it.next().unwrap();
        let right = unary(it)?;
        Ok(Expr::Unary {
            op: op.clone(),
            right: Box::new(right),
        })
    } else {
        primary(it)
    }
}

fn primary(it: &mut Peekable<Iter<Token>>) -> Result<Expr, ParsingError> {
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
            let expr = expression(it)?;
            expect_closing_paren(it)?;
            Ok(Expr::Grouping {
                expr: Box::new(expr),
            })
        }
        Some(eof) if matches!(eof.kind, TokenKind::Eof) => Err(ParsingError {
            message: String::from("Syntax error: expected primary expression, got EOF"),
            token: eof.clone(),
        }),
        Some(token) => Err(ParsingError {
            message: String::from("Syntax error: unexpected token"),
            token: token.clone(),
        }),
        None => panic!("Unexpected end of tokens. This is a bug."),
    }
}

fn expect_closing_paren(it: &mut Peekable<Iter<Token>>) -> Result<(), ParsingError> {
    match it.next() {
        Some(Token {
            kind: TokenKind::RightParen,
            ..
        }) => Ok(()),
        Some(not_close_paren) => Err(ParsingError {
            message: String::from("Syntax error: expected ')'"),
            token: not_close_paren.clone(),
        }),
        None => panic!("Unexpected end of tokens. This is a bug."),
    }
}

fn expect_semicolon(it: &mut Peekable<Iter<Token>>) -> Result<(), ParsingError> {
    match it.next() {
        Some(Token {
            kind: TokenKind::Semicolon,
            ..
        }) => Ok(()),
        Some(not_semicolon) => Err(ParsingError {
            message: String::from("Syntax error: expected ';'"),
            token: not_semicolon.clone(),
        }),
        None => panic!("Unexpected end of tokens. This is a bug."),
    }
}
