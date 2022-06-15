use super::lexer_type;
use super::lexer::*;
use super::instructions::*;

use std::iter::Peekable;

#[derive(Debug)]
pub enum ParserError {
    InvalidSyntax(LexerToken),
    UnmatchedParenthesis(LexerToken),
    ReachedEOF,
    ExpectedNameToBeWord(LexerToken),
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
    let mut instruction = LoispInstruction::new(LexerToken {kind: LexerTokenKind::Word, value: LexerTokenValue {string: String::new(), integer: 0}, location: LexerLocation::new(String::new())});
    if let Some(t) = lexer.peek() {
        instruction.token = (*t).clone();
    } else {
        return Err(ParserError::ReachedEOF)
    }

    if let Some(name) = lexer.next() {
        if name.kind != LexerTokenKind::Word {
            return Err(ParserError::ExpectedNameToBeWord(name.clone()))
        }

        instruction.kind = string_to_instruction_kind(name.clone().value.string)?;

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
                    value.instruction_return = parse_instruction(lexer)?;
                    instruction.parameters.push(value);
                }
                Word => {
                    return Err(ParserError::InvalidSyntax(next.clone()))
                }
                Integer => {
                    let mut value = LoispValue::new(next.clone());
                    value.integer = Some(next.value.integer);
                    instruction.parameters.push(value);
                }
            }
        }

        if !closed {
            return Err(ParserError::UnmatchedParenthesis(location.clone()))
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
            CloseParen => return Err(ParserError::UnmatchedParenthesis(x.clone())),
            Word => return Err(ParserError::InvalidSyntax(x.clone())),
            Integer => return Err(ParserError::InvalidSyntax(x.clone()))
        }
    }
    Ok(instructions)
}
