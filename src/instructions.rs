use super::types::*;
use super::ir::*;
use super::parser::*;
use super::lexer::*;

use std::fmt;
use std::io;
use std::collections::HashMap;

#[derive(Debug)]
pub enum LoispError {
    NotEnoughParameters(LexerToken),
    TooMuchParameters(LexerToken),
    MismatchedTypes(LexerToken),
    ParserError(ParserError),
    StandardError(io::Error),
    VariableNotFound(LexerToken),
    VariableRedefinition(LexerToken),
    CantAcceptNothing(LexerToken)
}

impl fmt::Display for LoispError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::NotEnoughParameters(token) => write!(f, "{}: ERROR: Not enough parameters for `{}`", token.location, token.value.string)?,
            Self::TooMuchParameters(token) => write!(f, "{}: ERROR: Too much parameters for `{}`", token.location, token.value.string)?,
            Self::MismatchedTypes(token) => write!(f, "{}: ERROR: Mismatched types on parameter for function `{}`", token.location, token.value.string)?,
            Self::ParserError(error) => write!(f, "{}", error)?,
            Self::StandardError(error) => write!(f, "ERROR: {:?}", error)?,
            Self::VariableNotFound(token) => write!(f, "{}: ERROR: Variable not found: `{}`", token.location, token.value.string)?,
            Self::VariableRedefinition(token) => write!(f, "{}: ERROR: Variable redefinition: `{}`", token.location, token.value.string)?,
            Self::CantAcceptNothing(token) => write!(f, "{}: ERROR: Can't accept value of type `Nothing` as parameter", token.location)?
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

#[derive(Debug, PartialEq, Clone)]
pub enum LoispInstructionType {
    Print,
    Plus,
    Minus,
    Multiplication,
    Division,
    Mod,
    Syscall,
    SetVar,
    GetVar,
    ChVar,
    Nop
}

#[derive(Debug, Clone)]
pub struct LoispValue {
    pub integer: Option<i64>,
    pub word: Option<String>,
    pub string: String,
    pub token: LexerToken,
    pub instruction_return: LoispInstruction
}

impl LoispValue {
    pub fn new(t: LexerToken) -> LoispValue {
        LoispValue {
            integer: None,
            word: None,
            string: String::new(),
            token: t.clone(),
            instruction_return: LoispInstruction::new(t.clone())
        }
    }

    pub fn is_instruction_return(&self) -> bool {
        self.integer.is_none() && self.string.len() == 0 && !self.instruction_return.is_empty()
    }

    pub fn datatype(&self, context: &mut LoispContext) -> Option<LoispDatatype> {
        use LoispDatatype::*;

        if !self.integer.is_none() {
            Some(Integer)
        } else if !self.word.is_none() {
            Some(Word)
        } else if self.string.len() != 0 {
            Some(String)
        } else if !self.instruction_return.is_empty() {
            let typee = self.instruction_return.return_type(context);
            Some(typee)
        } else {
            None
        }
    }

    pub fn size(&self, context: &mut LoispContext) -> usize {
        self.datatype(context).unwrap().size()
    }
}

#[derive(Debug, Clone)]
pub struct LoispVariable {
    pub id: usize,
    pub value: LoispValue
}

#[derive(Debug)]
pub struct LoispContext {
    pub variables: HashMap<String, LoispVariable>
}

impl LoispContext {
    pub fn new() -> LoispContext {
        LoispContext {
            variables: HashMap::new()
        }
    }
}

#[derive(Debug, Clone)]
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

    pub fn return_type(&self, context: &mut LoispContext) -> LoispDatatype {
        use LoispDatatype::*;
        match self.kind {
            LoispInstructionType::Print          => Nothing,
            LoispInstructionType::Nop            => Nothing,
            LoispInstructionType::Plus           => Integer,
            LoispInstructionType::Minus          => Integer,
            LoispInstructionType::Multiplication => Integer,
            LoispInstructionType::Division       => Integer,
            LoispInstructionType::Mod            => Integer,
            LoispInstructionType::Syscall        => Integer,
            LoispInstructionType::SetVar         => Nothing,
            LoispInstructionType::GetVar         => {
                let var = context.variables.get(self.parameters[0].word.as_ref().unwrap());
                if var.is_none() {
                    return Nothing
                } else {
                    return var.unwrap().value.clone().datatype(context).unwrap()
                }
            }
            LoispInstructionType::ChVar          => Nothing
        }
    }


    pub fn push_parameters(&self, ir: &mut IrProgram, context: &mut LoispContext) -> Result<(), LoispError> {
        for p in self.parameters.iter().rev() {
            if p.is_instruction_return() {
                if p.instruction_return.return_type(context) == LoispDatatype::Nothing {
                    return Err(LoispError::CantAcceptNothing(p.token.clone()));
                }
                p.instruction_return.to_ir(ir, context)?;
            } else {
                match p.datatype(context) {
                    Some(LoispDatatype::Integer) => ir.push(IrInstruction {kind: IrInstructionKind::PushInteger, operand: IrInstructionValue::new().integer(p.integer.unwrap())}),
                    Some(LoispDatatype::Word) => return Err(LoispError::ParserError(ParserError::InvalidSyntax(p.token.clone()))),
                    Some(LoispDatatype::String) => todo!("push strings"),
                    Some(LoispDatatype::Nothing) => return Err(LoispError::CantAcceptNothing(p.token.clone())),
                    None => panic!("unreachable")
                }
            }
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.kind == LoispInstructionType::Nop && self.parameters.len() == 0
    }

    pub fn to_ir(&self, ir: &mut IrProgram, context: &mut LoispContext) -> Result<(), LoispError> {
        use LoispInstructionType::*;

        match self.kind {
            Print => {
                self.push_parameters(ir, context)?;
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if self.parameters[0].datatype(context).unwrap() == LoispDatatype::Nothing {
                    return Err(LoispError::MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Print, operand: IrInstructionValue::new()});
            }
            Plus => {
                self.push_parameters(ir, context)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer) {
                    return Err(LoispError::MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Plus, operand: IrInstructionValue::new()});
            }
            Minus => {
                self.push_parameters(ir, context)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer) {
                    return Err(LoispError::MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Minus, operand: IrInstructionValue::new()});
            }
            Multiplication => {
                self.push_parameters(ir, context)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer) {
                    return Err(LoispError::MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Multiplication, operand: IrInstructionValue::new()});
            }
            Division => {
                self.push_parameters(ir, context)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer) {
                    return Err(LoispError::MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Division, operand: IrInstructionValue::new()});
            }
            Mod => {
                self.push_parameters(ir, context)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer) {
                    return Err(LoispError::MismatchedTypes(self.token.clone()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Mod, operand: IrInstructionValue::new()});
            }
            Syscall => {
                self.push_parameters(ir, context)?;
                if self.parameters.len() > 6 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                for v in &self.parameters {
                    if v.datatype(context).unwrap() != LoispDatatype::Integer {
                        return Err(LoispError::MismatchedTypes(self.token.clone()))
                    }
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Syscall, operand: IrInstructionValue::new().integer(self.parameters.len() as i64)});
            }
            SetVar => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()))
                }

                if self.parameters[1].datatype(context).unwrap() == LoispDatatype::Word {
                    return Err(LoispError::ParserError(ParserError::InvalidSyntax(self.token.clone())))
                }

                let variable = LoispVariable {
                    id: context.variables.len(),
                    value: self.parameters[1].clone()
                };

                if let Some(_) = context.variables.get(&self.parameters[0].clone().word.unwrap()) {
                    return Err(LoispError::VariableRedefinition(self.parameters[0].token.clone()))
                }

                context.variables.insert(self.parameters[0].clone().word.unwrap(), variable.clone());
                ir.push(IrInstruction {kind: IrInstructionKind::AllocMemory, operand: IrInstructionValue::new().integer(variable.clone().value.size(context) as i64)});

                {
                    let mut inst = LoispInstruction::new(self.token.clone());
                    let last: Vec<LoispValue> = vec![self.parameters.clone().last().unwrap().clone()];
                    inst.parameters = last;
                    inst.push_parameters(ir, context)?;
                }

                ir.push(IrInstruction {kind: IrInstructionKind::PushMemory, operand: IrInstructionValue::new().integer(variable.clone().id as i64)});
                value_size_as_store_instruction(variable.clone().value.datatype(context).unwrap().size(), ir);
            }
            GetVar => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()))
                }

                if let Some(var) = context.variables.get(self.parameters[0].word.as_ref().unwrap()) {
                    ir.push(IrInstruction {kind: IrInstructionKind::PushMemory, operand: IrInstructionValue::new().integer(var.id as i64)});
                    value_size_as_load_instruction(var.value.clone().size(context), ir);
                } else {
                    return Err(LoispError::VariableNotFound(self.parameters[0].token.clone()))
                }
            }
            ChVar => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()))
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()))
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()))
                }

                if let Some(var) = context.variables.get(self.parameters[0].word.as_ref().unwrap()) {
                    let parameter1 = self.parameters[1].clone();
                    let variable_value = var.value.clone();
                    if parameter1.datatype(context).unwrap() != variable_value.datatype(context).unwrap() {
                        return Err(LoispError::MismatchedTypes(self.token.clone()))
                    }
                } else {
                    return Err(LoispError::VariableNotFound(self.parameters[0].token.clone()))
                }

                let mutable_var = context.variables.get_mut(self.parameters[0].word.as_ref().unwrap()).unwrap();
                let var = mutable_var.clone();
                mutable_var.value = self.parameters[1].clone();

                {
                    let mut inst = LoispInstruction::new(self.token.clone());
                    let last: Vec<LoispValue> = vec![self.parameters.clone().last().unwrap().clone()];
                    inst.parameters = last;
                    inst.push_parameters(ir, context)?;
                }

                ir.push(IrInstruction {kind: IrInstructionKind::PushMemory, operand: IrInstructionValue::new().integer(var.id as i64)});
                value_size_as_store_instruction(var.clone().value.datatype(context).unwrap().size(), ir);
            }
            Nop => {}
        }
        Ok(())
    }
}
