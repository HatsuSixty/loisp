use super::lexer_type;
use super::lexer::*;
use super::instructions::*;

use std::iter::Peekable;

#[derive(Debug)]
pub enum ParserError {
    InvalidSyntax,
    UnmatchedParenthesis,
    ReachedEOF,
    ExpectedNameToBeWord,
    UnknownInstruction
}

pub fn string_to_instruction_kind(string: String) -> Result<LoispInstructionType, ParserError> {
    match string.as_str() {
        "print" => Ok(LoispInstructionType::Print),
        "+" => Ok(LoispInstructionType::Plus),
        _ => Err(ParserError::UnknownInstruction)
    }
}

pub fn parse_instruction(lexer: &mut lexer_type!()) -> Result<LoispInstruction, ParserError> {
    let mut instruction = LoispInstruction::new();

    if let Some(name) = lexer.next() {
        if name.kind != LexerTokenKind::Word {
            return Err(ParserError::ExpectedNameToBeWord)
        }

        instruction.kind = string_to_instruction_kind(name.value.string)?;

        let mut closed = false;

        while let Some(next) = lexer.next() {
            use LexerTokenKind::*;

            match next.kind {
                CloseParen => {
                    closed = true;
                    break;
                }
                OpenParen => {
                    let mut value = LoispValue::new();
                    value.instruction_return = parse_instruction(lexer)?;
                    instruction.parameters.push(value);
                }
                Word => {
                    return Err(ParserError::InvalidSyntax)
                }
                Integer => {
                    let mut value = LoispValue::new();
                    value.integer = Some(next.value.integer);
                    instruction.parameters.push(value);
                }
            }
        }

        if !closed {
            return Err(ParserError::UnmatchedParenthesis)
        }
    } else {
        return Err(ParserError::ReachedEOF)
    }

    Ok(instruction)
}

pub fn construct_instructions_from_tokens(lexer: &mut lexer_type!()) -> Result<Vec<LoispInstruction>, ParserError> {
    use LexerTokenKind::*;

    let mut instructions: Vec<LoispInstruction> = vec![];

    while let Some(x) = lexer.next() {
        match x.kind {
            OpenParen => {
                instructions.push(parse_instruction(lexer)?);
            }
            CloseParen => return Err(ParserError::UnmatchedParenthesis),
            Word => return Err(ParserError::InvalidSyntax),
            Integer => return Err(ParserError::InvalidSyntax)
        }
    }
    Ok(instructions)
}
