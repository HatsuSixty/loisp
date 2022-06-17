use super::types::*;
use super::ir::*;
use super::parser::*;
use super::lexer::*;

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum LoispError {
    NotEnoughParameters(LexerToken),
    TooMuchParameters(LexerToken),
    CantAcceptNothing(LexerToken),
    MismatchedTypes(LexerToken),
    ParserError(ParserError),
    StandardError(io::Error)
}

impl fmt::Display for LoispError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::NotEnoughParameters(token) => write!(f, "{}: ERROR: Not enough parameters for `{}`", token.location, token.value.string)?,
            Self::TooMuchParameters(token) => write!(f, "{}: ERROR: Too much parameters for `{}`", token.location, token.value.string)?,
            Self::CantAcceptNothing(token) => write!(f, "{}: ERROR: The function `{}` can't accept value of type `Nothing`", token.location, token.value.string)?,
            Self::MismatchedTypes(token) => write!(f, "{}: ERROR: Mismatched types on parameter for function `{}`", token.location, token.value.string)?,
            Self::ParserError(error) => write!(f, "{}", error)?,
            Self::StandardError(error) => write!(f, "ERROR: {:?}", error)?
        }
        Ok(())
    }
}

impl From<std::io::Error> for LoispError {
    fn from(e: std::io::Error) -> Self {
        Self::StandardError(e)
    }
}

impl From<ParserError> for LoispError {
    fn from(error: ParserError) -> Self {
        Self::ParserError(error)
    }
}

#[derive(Debug, PartialEq)]
pub enum LoispInstructionType {
    Print,
    Plus,
    Minus,
    Multiplication,
    Division,
    Mod,
    Nop
}

impl LoispInstructionType {
    pub fn return_type(&self) -> LoispDatatype {
        use LoispDatatype::*;
        match self {
            Self::Print          => Nothing,
            Self::Nop            => Nothing,
            Self::Plus           => Integer,
            Self::Minus          => Integer,
            Self::Multiplication => Integer,
            Self::Division       => Integer,
            Self::Mod            => Integer
        }
    }
}

#[derive(Debug)]
pub struct LoispValue {
    pub integer: Option<i64>,
    pub string: String,
    pub instruction_return: LoispInstruction
}

impl LoispValue {
    pub fn new(t: LexerToken) -> LoispValue {
        LoispValue {
            integer: None,
            string: String::new(),
            instruction_return: LoispInstruction::new(t)
        }
    }

    pub fn is_instruction_return(&self) -> bool {
        self.integer.is_none() && self.string.len() == 0 && !self.instruction_return.is_empty()
    }

    pub fn datatype(&self) -> Option<LoispDatatype> {
        use LoispDatatype::*;

        if !self.integer.is_none() {
            Some(Integer)
        } else if self.string.len() != 0 {
            Some(String)
        } else if !self.instruction_return.is_empty() {
            let typee = self.instruction_return.kind.return_type();
            Some(typee)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct LoispInstruction {
    pub kind: LoispInstructionType,
    pub parameters: Vec<LoispValue>,
    pub token: LexerToken
}

impl LoispInstruction {
    pub fn new(t: LexerToken) -> LoispInstruction {
        LoispInstruction {
            kind: LoispInstructionType::Nop,
            parameters: vec![],
            token: t
        }
    }

    pub fn is_empty(&self) -> bool {
        self.kind == LoispInstructionType::Nop && self.parameters.len() == 0
    }

    pub fn to_ir(&self, ir: &mut IrProgram) -> Result<(), LoispError> {
        use LoispInstructionType::*;
        use LoispError::*;

        for p in self.parameters.iter().rev() {
            if p.is_instruction_return() {
                p.instruction_return.to_ir(ir)?;
            } else {
                match p.datatype() {
                    Some(LoispDatatype::Integer) => ir.push(IrInstruction {kind: IrInstructionKind::PushInteger, operand: IrInstructionValue::new().integer(p.integer.unwrap())}),
                    Some(LoispDatatype::String) => todo!("push strings"),
                    _ => panic!("unreachable")
                }
            }
        }

        match self.kind {
            Print => {
                if self.parameters.len() < 1 {
                    return Err(NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 1 {
                    return Err(TooMuchParameters(self.token.clone()))
                }

                if self.parameters[0].datatype().unwrap() == LoispDatatype::Nothing {
                    return Err(CantAcceptNothing(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Print, operand: IrInstructionValue::new()});
            }
            Plus => {
                if self.parameters.len() < 2 {
                    return Err(NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype().unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype().unwrap() == LoispDatatype::Integer) {
                    return Err(MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Plus, operand: IrInstructionValue::new()});
            }
            Minus => {
                if self.parameters.len() < 2 {
                    return Err(NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype().unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype().unwrap() == LoispDatatype::Integer) {
                    return Err(MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Minus, operand: IrInstructionValue::new()});
            }
            Multiplication => {
                if self.parameters.len() < 2 {
                    return Err(NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype().unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype().unwrap() == LoispDatatype::Integer) {
                    return Err(MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Multiplication, operand: IrInstructionValue::new()});
            }
            Division => {
                if self.parameters.len() < 2 {
                    return Err(NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype().unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype().unwrap() == LoispDatatype::Integer) {
                    return Err(MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Division, operand: IrInstructionValue::new()});
            }
            Mod => {
                if self.parameters.len() < 2 {
                    return Err(NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype().unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype().unwrap() == LoispDatatype::Integer) {
                    return Err(MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Mod, operand: IrInstructionValue::new()});
            }
            Nop => {}
        }
        Ok(())
    }
}
