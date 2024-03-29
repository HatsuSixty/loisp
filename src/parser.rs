use super::instructions::*;
use super::lexer::*;
use super::lexer_type;

use std::fmt;
use std::iter::Peekable;

#[derive(Debug)]
pub enum ParserError {
    InvalidSyntax(LexerToken),
    UnmatchedParenthesis(LexerToken),
    ReachedEOF(LexerToken),
    ExpectedNameToBeWord(LexerToken),
    UnknownInstruction(LexerToken),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::InvalidSyntax(token) => write!(f, "{}: ERROR: Invalid syntax", token.location)?,
            Self::UnmatchedParenthesis(token) => {
                write!(f, "{}: ERROR: Unmatched parenthesis", token.location)?
            }
            Self::ExpectedNameToBeWord(token) => write!(
                f,
                "{}: ERROR: Expected word but got `{}`",
                token.location, token.value.string
            )?,
            Self::ReachedEOF(token) => {
                write!(f, "{}: ERROR: Reached EOF while parsing", token.location)?
            }
            Self::UnknownInstruction(token) => write!(
                f,
                "{}: ERROR: Unknown instruction: {}",
                token.location, token.value.string
            )?,
        }
        Ok(())
    }
}

pub fn token_to_instruction_kind(token: LexerToken) -> Result<LoispInstructionType, ParserError> {
    match token.value.string.as_str() {
        "print" => Ok(LoispInstructionType::Print),
        "+" => Ok(LoispInstructionType::Plus),
        "-" => Ok(LoispInstructionType::Minus),
        "*" => Ok(LoispInstructionType::Multiplication),
        "/" => Ok(LoispInstructionType::Division),
        "%" => Ok(LoispInstructionType::Mod),
        "syscall" => Ok(LoispInstructionType::Syscall),
        "setvar" => Ok(LoispInstructionType::SetVar),
        "getvar" => Ok(LoispInstructionType::GetVar),
        "chvar" => Ok(LoispInstructionType::ChVar),
        "while" => Ok(LoispInstructionType::While),
        "=" => Ok(LoispInstructionType::Equal),
        "!=" => Ok(LoispInstructionType::NotEqual),
        "<" => Ok(LoispInstructionType::Less),
        ">" => Ok(LoispInstructionType::Greater),
        "<=" => Ok(LoispInstructionType::LessEqual),
        ">=" => Ok(LoispInstructionType::GreaterEqual),
        "if" => Ok(LoispInstructionType::If),
        "block" => Ok(LoispInstructionType::Block),
        "ptrto" => Ok(LoispInstructionType::PtrTo),
        "load64" => Ok(LoispInstructionType::Load64),
        "store64" => Ok(LoispInstructionType::Store64),
        "load32" => Ok(LoispInstructionType::Load32),
        "store32" => Ok(LoispInstructionType::Store32),
        "load16" => Ok(LoispInstructionType::Load16),
        "store16" => Ok(LoispInstructionType::Store16),
        "load8" => Ok(LoispInstructionType::Load8),
        "store8" => Ok(LoispInstructionType::Store8),
        "alloc" => Ok(LoispInstructionType::Alloc),
        "getmem" => Ok(LoispInstructionType::GetMem),
        "castint" => Ok(LoispInstructionType::CastInt),
        "castptr" => Ok(LoispInstructionType::CastPointer),
        "&" => Ok(LoispInstructionType::And),
        "!" => Ok(LoispInstructionType::Not),
        "|" => Ok(LoispInstructionType::Or),
        "<<" => Ok(LoispInstructionType::ShiftLeft),
        ">>" => Ok(LoispInstructionType::ShiftRight),
        "macro" => Ok(LoispInstructionType::Macro),
        "expand" => Ok(LoispInstructionType::Expand),
        "pop" => Ok(LoispInstructionType::Pop),
        "include" => Ok(LoispInstructionType::Include),
        "defun" => Ok(LoispInstructionType::DefFun),
        "call" => Ok(LoispInstructionType::Call),
        "increment" => Ok(LoispInstructionType::Increment),
        "reset" => Ok(LoispInstructionType::Reset),
        "typeof" => Ok(LoispInstructionType::TypeOf),
        "argc" => Ok(LoispInstructionType::Argc),
        "argv" => Ok(LoispInstructionType::Argv),
        "envp" => Ok(LoispInstructionType::Envp),
        _ => Err(ParserError::UnknownInstruction(token.clone())),
    }
}

pub fn parse_instruction(
    lexer: &mut lexer_type!(),
    token: LexerToken,
) -> Result<LoispInstruction, ParserError> {
    let mut instruction = LoispInstruction::new(LexerToken {
        kind: LexerTokenKind::Word,
        value: LexerTokenValue {
            string: String::new(),
            integer: 0,
        },
        location: LexerLocation::new(String::new()),
    });
    if let Some(x) = lexer.peek() {
        instruction.token = x.clone();
    } else {
        return Err(ParserError::ReachedEOF(token.clone()));
    }

    if let Some(name) = lexer.next() {
        if name.kind != LexerTokenKind::Word {
            return Err(ParserError::ExpectedNameToBeWord(name.clone()));
        }

        instruction.kind = token_to_instruction_kind(name.clone())?;

        let mut closed = false;
        let location = name.clone();

        while let Some(next) = lexer.next() {
            use LexerTokenKind::*;

            match next.kind {
                CloseParen => {
                    closed = true;
                    break;
                }
                OpenParen => {
                    let mut value = LoispValue::new(next.clone());
                    value.instruction_return = Some(parse_instruction(lexer, next.clone())?);
                    instruction.parameters.push(value);
                }
                Word => {
                    let mut value = LoispValue::new(next.clone());
                    value.word = Some(next.value.string);
                    instruction.parameters.push(value);
                }
                Integer => {
                    let mut value = LoispValue::new(next.clone());
                    value.integer = Some(next.value.integer);
                    instruction.parameters.push(value);
                }
                String => {
                    let mut value = LoispValue::new(next.clone());
                    value.string = Some(next.value.string);
                    instruction.parameters.push(value);
                }
            }
        }

        if !closed {
            return Err(ParserError::UnmatchedParenthesis(location.clone()));
        }
    } else {
        return Err(ParserError::ReachedEOF(token));
    }

    Ok(instruction)
}

pub fn construct_instructions_from_tokens(
    lexer: &mut lexer_type!(),
) -> Result<Vec<LoispInstruction>, ParserError> {
    use LexerTokenKind::*;

    let mut instructions: Vec<LoispInstruction> = vec![];

    while let Some(x) = lexer.next() {
        match x.kind {
            OpenParen => {
                instructions.push(parse_instruction(lexer, x)?);
            }
            CloseParen => return Err(ParserError::UnmatchedParenthesis(x.clone())),
            Word => return Err(ParserError::InvalidSyntax(x.clone())),
            Integer => return Err(ParserError::InvalidSyntax(x.clone())),
            String => return Err(ParserError::InvalidSyntax(x.clone())),
        }
    }
    Ok(instructions)
}
