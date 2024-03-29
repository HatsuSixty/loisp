use std::fmt;
use std::iter::Peekable;

#[derive(PartialEq, Debug, Clone)]
pub enum LexerTokenKind {
    OpenParen,
    CloseParen,
    Word,
    Integer,
    String,
}

pub fn is_special_token(c: char) -> bool {
    c == '(' || c == ')' || c == ' ' || c == '\n' || c == '\r' || c == '#' || c == '"'
}

#[derive(Debug, Clone)]
pub struct LexerTokenValue {
    pub integer: i64,
    pub string: String,
}

impl LexerTokenValue {
    pub fn from_string(s: String) -> LexerTokenValue {
        LexerTokenValue {
            integer: 0,
            string: s,
        }
    }

    pub fn from_int(i: i64) -> LexerTokenValue {
        LexerTokenValue {
            integer: i,
            string: "".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LexerLocation {
    pub f: String,
    pub r: i64,
    pub c: i64,
}

impl fmt::Display for LexerLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}:{}:{}", self.f, self.r, self.c)?;
        Ok(())
    }
}

impl LexerLocation {
    pub fn new(f: String) -> LexerLocation {
        LexerLocation { f: f, r: 1, c: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct LexerToken {
    pub kind: LexerTokenKind,
    pub value: LexerTokenValue,
    pub location: LexerLocation,
}

pub struct Lexer<Chars: Iterator<Item = char>> {
    pub chars: Peekable<Chars>,
    pub location: LexerLocation,
}

impl<Chars: Iterator<Item = char>> Lexer<Chars> {
    pub fn from_chars(chars: Chars, f: String) -> Self {
        Self {
            chars: chars.peekable(),
            location: LexerLocation::new(f),
        }
    }

    pub fn ignore_whitespaces_and_comments(&mut self) {
        while let Some(c) = self.chars.next_if(|x| x.is_whitespace()) {
            self.advance_location(c);
        }

        if let Some(x) = self.chars.peek() {
            if *x == '#' {
                while let Some(it) = self.chars.next() {
                    self.advance_location(it);
                    if it == '\n' || it == '\r' {
                        break;
                    }
                }
            } else if *x == '{' {
                while let Some(it) = self.chars.next() {
                    self.advance_location(it);
                    if it == '}' {
                        break;
                    }
                }
            } else {
                return;
            }
        } else {
            return;
        }

        self.ignore_whitespaces_and_comments();
    }

    pub fn advance_location(&mut self, c: char) {
        self.location.c += 1;
        if c == '\n' {
            self.location.r += 1;
            self.location.c = 0;
        }
    }
}

impl<Chars: Iterator<Item = char>> Iterator for Lexer<Chars> {
    type Item = LexerToken;

    fn next(&mut self) -> Option<LexerToken> {
        use LexerTokenKind::*;
        self.ignore_whitespaces_and_comments();

        if let Some(x) = self.chars.next() {
            let mut text = "".to_string();
            text.push(x);
            self.advance_location(x);
            match x {
                '(' => Some(LexerToken {
                    kind: OpenParen,
                    value: LexerTokenValue::from_string(text),
                    location: self.location.clone(),
                }),
                ')' => Some(LexerToken {
                    kind: CloseParen,
                    value: LexerTokenValue::from_string(text),
                    location: self.location.clone(),
                }),
                '"' => {
                    let mut string = "".to_string();
                    while let Some(x) = self.chars.next_if(|x| *x != '"') {
                        string.push(x);
                        if x == '\\' {
                            if let Some(a) = self.chars.next() {
                                string.push(a);
                            }
                        }
                    }
                    if let Some(_) = self.chars.peek() {
                        self.chars.next();
                    } else {
                        eprintln!("ERROR: Reached EOF while parsing string");
                        std::process::exit(1);
                    }
                    Some(LexerToken {
                        kind: LexerTokenKind::String,
                        value: LexerTokenValue::from_string(string),
                        location: self.location.clone(),
                    })
                }
                _ => {
                    while let Some(x) = self.chars.next_if(|x| !is_special_token(*x)) {
                        text.push(x);
                        self.advance_location(x);
                    }

                    let parsed = text.parse::<i64>();
                    let mut location = self.location.clone();
                    location.c -= (text.len() as i64) - 1;
                    if let Err(_) = parsed {
                        Some(LexerToken {
                            kind: Word,
                            value: LexerTokenValue::from_string(text),
                            location: location.clone(),
                        })
                    } else {
                        Some(LexerToken {
                            kind: Integer,
                            value: LexerTokenValue::from_int(parsed.unwrap()),
                            location: location.clone(),
                        })
                    }
                }
            }
        } else {
            None
        }
    }
}

#[macro_export]
macro_rules! lexer_type {
    () => {
        Peekable<impl Iterator<Item=LexerToken>>
    }
}
