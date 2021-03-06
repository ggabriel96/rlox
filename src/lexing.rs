use std::iter::Peekable;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

#[derive(Clone, Debug)]
pub enum TokenKind {
    // Single-character tokens
    Comma,
    Dot,
    LeftBrace,
    LeftParen,
    Minus,
    Plus,
    RightBrace,
    RightParen,
    Semicolon,
    Slash,
    Star,

    // Operators
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    Number,
    String,

    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // Misc and whitespace
    Comment,
    Eof,
    NewLine,
    Whitespace,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Clone, Debug)]
pub struct Loc {
    pub line_begin: usize,
    pub line_end: usize,
}

impl Loc {
    pub fn is_single(&self) -> bool {
        self.line_begin == self.line_end
    }

    pub fn offset(&self) -> usize {
        self.line_end - self.line_begin
    }

    pub fn single(number: usize) -> Loc {
        Loc {
            line_begin: number,
            line_end: number,
        }
    }
}

#[derive(Debug)]
pub struct LexingError {
    message: String,
    lexeme: Option<String>,
    loc: Loc,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub loc: Loc,
}

#[derive(Debug)]
pub struct Scanner {
    source: String,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner { source: source }
    }

    pub fn scan(&mut self) -> Result<Vec<Token>, LexingError> {
        let mut current_line: usize = 1;
        let mut graphemes_iter = self.source.graphemes(true).peekable();
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            match self.parse_token(&mut graphemes_iter, current_line)? {
                Token {
                    kind: TokenKind::Whitespace,
                    ..
                } => (),
                Token {
                    kind: TokenKind::Comment,
                    ..
                } => self.consume_line(&mut graphemes_iter),
                Token {
                    kind: TokenKind::NewLine,
                    ..
                } => current_line += 1,
                tok => {
                    if !tok.loc.is_single() {
                        current_line += tok.loc.offset();
                    }
                    let kind = tok.kind.clone();
                    tokens.push(tok);
                    if matches!(kind, TokenKind::Eof) {
                        break;
                    }
                }
            }
        }
        Ok(tokens)
    }

    fn parse_identifier(
        &self,
        graphemes_iter: &mut Peekable<Graphemes>,
        first_char: &str,
        current_line: usize,
    ) -> Token {
        let mut string = vec![String::from(first_char)];
        while let Some(g) = graphemes_iter.peek() {
            if !Scanner::is_ident_trailing(g) {
                break;
            }
            string.push(String::from(graphemes_iter.next().unwrap()));
        }
        let string = string.concat();
        Scanner::keyword_or_identifier_token(string.as_str(), current_line)
    }

    fn parse_number_literal(
        &self,
        graphemes_iter: &mut Peekable<Graphemes>,
        first_digit: &str,
        current_line: usize,
    ) -> Result<Token, LexingError> {
        let mut string = vec![String::from(first_digit)];
        let mut has_point = first_digit == ".";
        loop {
            let literal = match graphemes_iter.peek() {
                Some(g) if Scanner::is_digit(g) => graphemes_iter.next().unwrap(),
                Some(&".") => {
                    if has_point {
                        return Err(LexingError {
                            message: String::from(
                                "Unexpected additional point while parsing number at line {}",
                            ),
                            lexeme: Some(String::from(".")),
                            loc: Loc::single(current_line),
                        });
                    }
                    has_point = true;
                    graphemes_iter.next().unwrap()
                }
                _ => break,
            };
            string.push(String::from(literal));
        }
        let string = string.concat();
        Ok(Token {
            kind: TokenKind::Number,
            lexeme: string.clone(),
            literal: Some(LiteralValue::Number(string.parse::<f64>().unwrap())),
            loc: Loc::single(current_line),
        })
    }

    fn parse_str_literal(
        &self,
        graphemes_iter: &mut Peekable<Graphemes>,
        line_begin: usize,
    ) -> Result<Token, LexingError> {
        let mut line_current = line_begin;
        let mut string: Vec<String> = Vec::new();
        loop {
            let grapheme1 = graphemes_iter.next();
            let grapheme2 = graphemes_iter.peek();
            let literal = match (grapheme1, grapheme2) {
                (Some("\n"), None) | (None, _) => {
                    return Err(LexingError {
                        message: String::from("Unexpected EOF in unterminated string"),
                        lexeme: None,
                        loc: Loc::single(line_current),
                    })
                }
                (Some("\\"), Some(&"\"")) => {
                    graphemes_iter.next();
                    "\\\""
                }
                (Some("\n"), _) => {
                    line_current += 1;
                    "\n"
                }
                (Some("\""), _) => {
                    break;
                }
                (Some(l), _) => l,
            };
            string.push(String::from(literal));
        }
        let string = string.concat();
        Ok(Token {
            kind: TokenKind::String,
            lexeme: [String::from("\""), string.clone(), String::from("\"")].concat(),
            literal: Some(LiteralValue::String(string)),
            loc: Loc {
                line_begin: line_begin,
                line_end: line_current,
            },
        })
    }

    fn parse_token(
        &self,
        graphemes_iter: &mut Peekable<Graphemes>,
        current_line: usize,
    ) -> Result<Token, LexingError> {
        let grapheme1 = graphemes_iter.next();
        let grapheme2 = graphemes_iter.peek();
        let token = match grapheme1 {
            None => Token {
                kind: TokenKind::Eof,
                lexeme: String::from("\0"),
                literal: None,
                loc: Loc::single(current_line),
            },
            Some("\"") => self.parse_str_literal(graphemes_iter, current_line)?,
            Some(l) if Scanner::is_digit(l) => {
                self.parse_number_literal(graphemes_iter, l, current_line)?
            }
            l @ Some(".") => {
                if grapheme2.is_some() && Scanner::is_digit(grapheme2.unwrap()) {
                    self.parse_number_literal(graphemes_iter, l.unwrap(), current_line)?
                } else {
                    Token {
                        kind: TokenKind::Dot,
                        lexeme: String::from(l.unwrap()),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                }
            }
            Some(l) if Scanner::is_ident_start(l) => {
                self.parse_identifier(graphemes_iter, l, current_line)
            }
            l @ Some(" ") | l @ Some("\r") | l @ Some("\t") => Token {
                kind: TokenKind::Whitespace,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some("(") => Token {
                kind: TokenKind::LeftParen,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some(")") => Token {
                kind: TokenKind::RightParen,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some("{") => Token {
                kind: TokenKind::LeftBrace,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some("}") => Token {
                kind: TokenKind::RightBrace,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some(",") => Token {
                kind: TokenKind::Comma,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some("-") => Token {
                kind: TokenKind::Minus,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some("+") => Token {
                kind: TokenKind::Plus,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some(";") => Token {
                kind: TokenKind::Semicolon,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some("*") => Token {
                kind: TokenKind::Star,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some("\n") => Token {
                kind: TokenKind::NewLine,
                lexeme: String::from(l.unwrap()),
                literal: None,
                loc: Loc::single(current_line),
            },
            l @ Some("!") => {
                if grapheme2 == Some(&&"=") {
                    graphemes_iter.next();
                    Token {
                        kind: TokenKind::BangEqual,
                        lexeme: String::from("!="),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                } else {
                    Token {
                        kind: TokenKind::Bang,
                        lexeme: String::from(l.unwrap()),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                }
            }
            l @ Some("=") => {
                if grapheme2 == Some(&&"=") {
                    graphemes_iter.next();
                    Token {
                        kind: TokenKind::EqualEqual,
                        lexeme: String::from("=="),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                } else {
                    Token {
                        kind: TokenKind::Equal,
                        lexeme: String::from(l.unwrap()),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                }
            }
            l @ Some("<") => {
                if grapheme2 == Some(&&"=") {
                    graphemes_iter.next();
                    Token {
                        kind: TokenKind::LessEqual,
                        lexeme: String::from("<="),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                } else {
                    Token {
                        kind: TokenKind::Less,
                        lexeme: String::from(l.unwrap()),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                }
            }
            l @ Some(">") => {
                if grapheme2 == Some(&&"=") {
                    graphemes_iter.next();
                    Token {
                        kind: TokenKind::GreaterEqual,
                        lexeme: String::from(">="),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                } else {
                    Token {
                        kind: TokenKind::Greater,
                        lexeme: String::from(l.unwrap()),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                }
            }
            l @ Some("/") => {
                if grapheme2 == Some(&&"/") {
                    graphemes_iter.next();
                    Token {
                        kind: TokenKind::Comment,
                        lexeme: String::from("//"),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                } else {
                    Token {
                        kind: TokenKind::Slash,
                        lexeme: String::from(l.unwrap()),
                        literal: None,
                        loc: Loc::single(current_line),
                    }
                }
            }
            Some(uk) => {
                return Err(LexingError {
                    message: String::from("Unknown token"),
                    lexeme: Some(String::from(uk)),
                    loc: Loc::single(current_line),
                })
            }
        };
        Ok(token)
    }

    fn consume_line(&self, graphemes_iter: &mut Peekable<Graphemes>) {
        while graphemes_iter.next() != None && graphemes_iter.peek() != Some(&"\n") {}
    }

    fn keyword_or_identifier_token(grapheme: &str, current_line: usize) -> Token {
        let kind = match grapheme {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        };
        let literal = match kind {
            TokenKind::False => Some(LiteralValue::Bool(false)),
            TokenKind::True => Some(LiteralValue::Bool(true)),
            _ => None,
        };
        Token {
            kind: kind,
            lexeme: String::from(grapheme),
            literal: literal,
            loc: Loc::single(current_line),
        }
    }

    fn is_digit(grapheme: &str) -> bool {
        match grapheme {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => true,
            _ => false,
        }
    }

    fn is_ident_start(grapheme: &str) -> bool {
        match grapheme {
            "_" | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M"
            | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" | "a"
            | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o"
            | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" => true,
            _ => false,
        }
    }

    fn is_ident_trailing(grapheme: &str) -> bool {
        Scanner::is_digit(grapheme) || Scanner::is_ident_start(grapheme)
    }
}
