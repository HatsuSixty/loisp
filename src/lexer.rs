use std::iter::Peekable;

#[derive(PartialEq, Debug)]
pub enum LexerTokenKind {
    OpenParen,
    CloseParen,
    Word,
    Integer
}

#[derive(Debug)]
pub struct LexerTokenValue {
    pub integer: i64,
    pub string: String
}

impl LexerTokenValue {
    pub fn from_string(s: String) -> LexerTokenValue {
        LexerTokenValue {
            integer: 0,
            string: s
        }
    }

    pub fn from_int(i: i64) -> LexerTokenValue {
        LexerTokenValue {
            integer: i,
            string: "".to_string()
        }
    }
}

#[derive(Debug)]
pub struct LexerToken {
    pub kind: LexerTokenKind,
    pub value: LexerTokenValue
}

pub struct Lexer<Chars: Iterator<Item=char>> {
    pub chars: Peekable<Chars>
}

impl<Chars: Iterator<Item=char>> Lexer<Chars> {
    pub fn from_chars(chars: Chars) -> Self {
        Self { chars: chars.peekable() }
    }
}

impl<Chars: Iterator<Item=char>> Iterator for Lexer<Chars> {
    type Item = LexerToken;
    fn next(&mut self) -> Option<LexerToken> {
        use LexerTokenKind::*;
        while let Some(_) = self.chars.next_if(|x| x.is_whitespace()) {}

        if let Some(x) = self.chars.next() {
            let mut text = "".to_string();
            text.push(x);
            match x {
                '(' => Some(LexerToken {kind: OpenParen, value: LexerTokenValue::from_string(text)}),
                ')' => Some(LexerToken {kind: CloseParen, value: LexerTokenValue::from_string(text)}),
                _   => {
                    while let Some(x) = self.chars.next_if(|x| x.is_alphanumeric()) {
                        text.push(x);
                    }

                    let parsed = text.parse::<i64>();
                    if let Err(_) = parsed {
                        Some(LexerToken {kind: Word, value: LexerTokenValue::from_string(text)})
                    } else {
                        Some(LexerToken {kind: Integer, value: LexerTokenValue::from_int(parsed.unwrap())})
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
