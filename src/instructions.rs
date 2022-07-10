use super::ir::*;
use super::lexer::*;
use super::parser::*;
use super::types::*;

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::Path;

static DEFAULT_SEARCH_PATHS: [&str; 4] = [".", "..", "./std", "../std"];

#[derive(Debug)]
pub enum LoispError {
    NotEnoughParameters(LexerToken),
    TooMuchParameters(LexerToken),
    MismatchedTypes(LexerToken),
    ParserError(ParserError),
    StandardError(io::Error),
    VariableNotFound(LexerToken),
    VariableRedefinition(LexerToken),
    NoDeclarationsInLoops(LexerToken),
    NoDeclarationsInMacros(LexerToken),
    MemoryRedefinition(LexerToken),
    MemoryNotFound(LexerToken),
    CantEvaluateAtCompileTime(LexerToken),
    MacroNotFound(LexerToken),
    MacroRedefinition(LexerToken),
    NoJumpsInMacros(LexerToken),
    FunctionRedefinition(LexerToken),
    FunctionNotFound(LexerToken),
    NoDeclarationsInFunctions(LexerToken),
}

impl fmt::Display for LoispError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::NotEnoughParameters(token) => write!(
                f,
                "{}: ERROR: Not enough parameters for `{}`",
                token.location, token.value.string
            )?,
            Self::TooMuchParameters(token) => write!(
                f,
                "{}: ERROR: Too much parameters for `{}`",
                token.location, token.value.string
            )?,
            Self::MismatchedTypes(token) => write!(
                f,
                "{}: ERROR: Mismatched types on parameter for function `{}`",
                token.location, token.value.string
            )?,
            Self::ParserError(error) => write!(f, "{}", error)?,
            Self::StandardError(error) => write!(f, "ERROR: {:?}", error)?,
            Self::VariableNotFound(token) => write!(
                f,
                "{}: ERROR: Variable not found: `{}`",
                token.location, token.value.string
            )?,
            Self::VariableRedefinition(token) => write!(
                f,
                "{}: ERROR: Variable redefinition: `{}`",
                token.location, token.value.string
            )?,
            Self::NoDeclarationsInLoops(token) => write!(
                f,
                "{}: ERROR: Declarations in loops are not allowed",
                token.location
            )?,
            Self::NoDeclarationsInMacros(token) => write!(
                f,
                "{}: ERROR: Declarations inside macros are not allowed",
                token.location
            )?,
            Self::NoDeclarationsInFunctions(token) => write!(
                f,
                "{}: ERROR: This kind of declaration inside functions is not allowed",
                token.location
            )?,
            Self::MemoryRedefinition(token) => write!(
                f,
                "{}: ERROR: Memory redefinition: `{}`",
                token.location, token.value.string
            )?,
            Self::MemoryNotFound(token) => write!(
                f,
                "{}: ERROR: Memory not found: `{}`",
                token.location, token.value.string
            )?,
            Self::CantEvaluateAtCompileTime(token) => write!(
                f,
                "{}: ERROR: Can't evaluate expression given as parameter for `{}` at compile time",
                token.location, token.value.string
            )?,
            Self::MacroNotFound(token) => write!(
                f,
                "{}: ERROR: Macro not found: `{}`",
                token.location, token.value.string
            )?,
            Self::MacroRedefinition(token) => write!(
                f,
                "{}: ERROR: Macro redefinition: `{}`",
                token.location, token.value.string
            )?,
            Self::FunctionRedefinition(token) => write!(
                f,
                "{}: ERROR: Function redefinition: `{}`",
                token.location, token.value.string
            )?,
            Self::NoJumpsInMacros(token) => write!(
                f,
                "{}: ERROR: Macros should not contain instructions that perform jumps",
                token.location
            )?,
            Self::FunctionNotFound(token) => write!(
                f,
                "{}: ERROR: Function not found: `{}`",
                token.location, token.value.string
            )?,
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
    Nop,
    While,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    If,
    Block,
    PtrTo,
    Load64,
    Store64,
    Load32,
    Store32,
    Load16,
    Store16,
    Load8,
    Store8,
    Alloc,
    GetMem,
    CastPointer,
    CastInt,
    ShiftLeft,
    ShiftRight,
    Or,
    And,
    Not,
    Macro,
    Expand,
    Pop,
    Include,
    DefFun,
    Call,
}

#[derive(Debug, Clone)]
pub struct LoispValue {
    pub integer: Option<i64>,
    pub word: Option<String>,
    pub string: String,
    pub token: LexerToken,
    pub instruction_return: LoispInstruction,
}

impl LoispValue {
    pub fn new(t: LexerToken) -> LoispValue {
        LoispValue {
            integer: None,
            word: None,
            string: String::new(),
            token: t.clone(),
            instruction_return: LoispInstruction::new(t.clone()),
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
pub struct LoispFunction {
    pub addr: usize,
    pub typ: LoispDatatype,
}

#[derive(Debug, Clone)]
pub struct LoispMacro {
    pub id: usize,
    pub program: IrProgram,
}

#[derive(Debug, Clone)]
pub struct LoispVariable {
    pub id: usize,
    pub value: LoispValue,
}

#[derive(Debug, Clone)]
pub struct LoispMemory {
    pub id: usize,
    pub alloc: usize,
}

#[derive(Debug)]
pub struct LoispContext {
    pub variables: HashMap<String, LoispVariable>,
    pub memories: HashMap<String, LoispMemory>,
    pub macros: HashMap<String, LoispMacro>,
    pub functions: HashMap<String, LoispFunction>,
    pub local_memories: HashMap<String, LoispMemory>,
    pub local_variables: HashMap<String, LoispVariable>,
    pub inside_fun: bool,
}

impl LoispContext {
    pub fn new() -> LoispContext {
        LoispContext {
            variables: HashMap::new(),
            memories: HashMap::new(),
            macros: HashMap::new(),
            functions: HashMap::new(),
            local_memories: HashMap::new(),
            local_variables: HashMap::new(),
            inside_fun: false,
        }
    }

    pub fn get_memory_count(&self) -> usize {
        self.local_memories.len() + self.memories.len()
    }

    pub fn get_variable_count(&self) -> usize {
        self.local_variables.len() + self.variables.len()
    }
}

pub fn value_size_as_store_instruction(s: usize, ir: &mut IrProgram) {
    match s {
        1 => ir_push(
            IrInstruction {
                kind: IrInstructionKind::Store8,
                operand: IrInstructionValue::new(),
            },
            ir,
        ),
        2 => ir_push(
            IrInstruction {
                kind: IrInstructionKind::Store16,
                operand: IrInstructionValue::new(),
            },
            ir,
        ),
        4 => ir_push(
            IrInstruction {
                kind: IrInstructionKind::Store32,
                operand: IrInstructionValue::new(),
            },
            ir,
        ),
        8 => ir_push(
            IrInstruction {
                kind: IrInstructionKind::Store64,
                operand: IrInstructionValue::new(),
            },
            ir,
        ),
        _ => panic!("unreachable"),
    }
}

pub fn value_size_as_load_instruction(s: usize, ir: &mut IrProgram) {
    match s {
        1 => ir_push(
            IrInstruction {
                kind: IrInstructionKind::Load8,
                operand: IrInstructionValue::new(),
            },
            ir,
        ),
        2 => ir_push(
            IrInstruction {
                kind: IrInstructionKind::Load16,
                operand: IrInstructionValue::new(),
            },
            ir,
        ),
        4 => ir_push(
            IrInstruction {
                kind: IrInstructionKind::Load32,
                operand: IrInstructionValue::new(),
            },
            ir,
        ),
        8 => ir_push(
            IrInstruction {
                kind: IrInstructionKind::Load64,
                operand: IrInstructionValue::new(),
            },
            ir,
        ),
        _ => panic!("unreachable"),
    }
}

pub fn ir_push(inst: IrInstruction, ir: &mut IrProgram) {
    ir.push(inst);
}

pub fn push_value(
    p: LoispValue,
    ir: &mut IrProgram,
    context: &mut LoispContext,
) -> Result<(), LoispError> {
    if p.is_instruction_return() {
        p.instruction_return.to_ir(ir, context)?;
    } else {
        match p.datatype(context) {
            Some(LoispDatatype::Integer) => ir_push(
                IrInstruction {
                    kind: IrInstructionKind::PushInteger,
                    operand: IrInstructionValue::new().integer(p.integer.unwrap()),
                },
                ir,
            ),
            Some(LoispDatatype::Pointer) => ir_push(
                IrInstruction {
                    kind: IrInstructionKind::PushInteger,
                    operand: IrInstructionValue::new().integer(p.integer.unwrap()),
                },
                ir,
            ),
            Some(LoispDatatype::Word) => {
                return Err(LoispError::ParserError(ParserError::InvalidSyntax(
                    p.token.clone(),
                )))
            }
            Some(LoispDatatype::String) => ir_push(
                IrInstruction {
                    kind: IrInstructionKind::PushString,
                    operand: IrInstructionValue::new().string(p.string),
                },
                ir,
            ),
            Some(LoispDatatype::Nothing) => {}
            None => panic!("unreachable"),
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct LoispInstruction {
    pub kind: LoispInstructionType,
    pub parameters: Vec<LoispValue>,
    pub token: LexerToken,
}

impl LoispInstruction {
    pub fn new(t: LexerToken) -> LoispInstruction {
        LoispInstruction {
            kind: LoispInstructionType::Nop,
            parameters: vec![],
            token: t,
        }
    }

    pub fn return_type(&self, context: &mut LoispContext) -> LoispDatatype {
        use LoispDatatype::*;
        match self.kind {
            LoispInstructionType::Print => Nothing,
            LoispInstructionType::Nop => Nothing,
            LoispInstructionType::Plus => Integer,
            LoispInstructionType::Minus => Integer,
            LoispInstructionType::Multiplication => Integer,
            LoispInstructionType::Division => Integer,
            LoispInstructionType::Mod => Integer,
            LoispInstructionType::Syscall => Integer,
            LoispInstructionType::SetVar => Nothing,
            LoispInstructionType::GetVar => {
                let var = context
                    .variables
                    .get(self.parameters[0].word.as_ref().unwrap());
                let local_var = context
                    .local_variables
                    .get(self.parameters[0].word.as_ref().unwrap());
                if var.is_none() {
                    if local_var.is_none() {
                        return Nothing;
                    }
                    return local_var.unwrap().value.clone().datatype(context).unwrap();
                } else {
                    return var.unwrap().value.clone().datatype(context).unwrap();
                }
            }
            LoispInstructionType::ChVar => Nothing,
            LoispInstructionType::While => Nothing,
            LoispInstructionType::Equal => Integer,
            LoispInstructionType::NotEqual => Integer,
            LoispInstructionType::If => Nothing,
            LoispInstructionType::Block => Nothing,
            LoispInstructionType::Less => Integer,
            LoispInstructionType::Greater => Integer,
            LoispInstructionType::LessEqual => Integer,
            LoispInstructionType::GreaterEqual => Integer,
            LoispInstructionType::PtrTo => Pointer,
            LoispInstructionType::Load64 => Integer,
            LoispInstructionType::Store64 => Nothing,
            LoispInstructionType::Load32 => Integer,
            LoispInstructionType::Store32 => Nothing,
            LoispInstructionType::Load16 => Integer,
            LoispInstructionType::Store16 => Nothing,
            LoispInstructionType::Load8 => Integer,
            LoispInstructionType::Store8 => Nothing,
            LoispInstructionType::Alloc => Nothing,
            LoispInstructionType::GetMem => Pointer,
            LoispInstructionType::CastInt => Integer,
            LoispInstructionType::CastPointer => Pointer,
            LoispInstructionType::ShiftLeft => Integer,
            LoispInstructionType::ShiftRight => Integer,
            LoispInstructionType::Or => Integer,
            LoispInstructionType::And => Integer,
            LoispInstructionType::Not => Integer,
            LoispInstructionType::Macro => Nothing,
            LoispInstructionType::Expand => {
                let maccro = context
                    .macros
                    .get(self.parameters[0].word.as_ref().unwrap());
                if maccro.is_none() {
                    return Nothing;
                } else {
                    if let Some(last) = maccro.unwrap().program.instructions.last() {
                        return last.get_loisp_datatype();
                    } else {
                        return Nothing;
                    }
                }
            }
            LoispInstructionType::Pop => Nothing,
            LoispInstructionType::Include => Nothing,
            LoispInstructionType::DefFun => Nothing,
            LoispInstructionType::Call => {
                if let Some(function) = context
                    .functions
                    .get(self.parameters[0].word.as_ref().unwrap())
                {
                    return function.clone().typ;
                } else {
                    return Nothing;
                }
            }
        }
    }

    pub fn push_parameters(
        &self,
        ir: &mut IrProgram,
        context: &mut LoispContext,
        reverse: bool,
    ) -> Result<(), LoispError> {
        if !reverse {
            for p in self.parameters.iter() {
                push_value(p.clone(), ir, context)?;
            }
        } else {
            for p in self.parameters.iter().rev() {
                push_value(p.clone(), ir, context)?;
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
                self.push_parameters(ir, context, true)?;
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() == LoispDatatype::Nothing {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Print,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Plus => {
                self.push_parameters(ir, context, true)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                    && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer)
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Plus,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Minus => {
                self.push_parameters(ir, context, true)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                    && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer)
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Minus,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Multiplication => {
                self.push_parameters(ir, context, true)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                    && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer)
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Multiplication,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Division => {
                self.push_parameters(ir, context, true)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                    && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer)
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Division,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Mod => {
                self.push_parameters(ir, context, true)?;
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if !(self.parameters[0].datatype(context).unwrap() == LoispDatatype::Integer
                    && self.parameters[1].datatype(context).unwrap() == LoispDatatype::Integer)
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Mod,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Syscall => {
                self.push_parameters(ir, context, true)?;
                if self.parameters.len() > 6 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                for v in &self.parameters {
                    if v.datatype(context).unwrap() != LoispDatatype::Integer {
                        return Err(LoispError::MismatchedTypes(self.token.clone()));
                    }
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Syscall,
                        operand: IrInstructionValue::new().integer(self.parameters.len() as i64),
                    },
                    ir,
                );
            }
            SetVar => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if self.parameters[1].datatype(context).unwrap() == LoispDatatype::Word {
                    return Err(LoispError::ParserError(ParserError::InvalidSyntax(
                        self.token.clone(),
                    )));
                }

                let variable = LoispVariable {
                    id: context.get_variable_count(),
                    value: self.parameters[1].clone(),
                };

                if let Some(_) = context
                    .variables
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    return Err(LoispError::VariableRedefinition(
                        self.parameters[0].token.clone(),
                    ));
                }

                if let Some(_) = context
                    .local_variables
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    return Err(LoispError::VariableRedefinition(
                        self.parameters[0].token.clone(),
                    ));
                }

                if context.inside_fun {
                    context
                        .local_variables
                        .insert(self.parameters[0].clone().word.unwrap(), variable.clone());
                } else {
                    context
                        .variables
                        .insert(self.parameters[0].clone().word.unwrap(), variable.clone());
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::AllocVariable,
                        operand: IrInstructionValue::new()
                            .integer(variable.clone().value.size(context) as i64),
                    },
                    ir,
                );

                push_value(self.parameters.clone().last().unwrap().clone(), ir, context)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::PushVariable,
                        operand: IrInstructionValue::new().integer(variable.clone().id as i64),
                    },
                    ir,
                );
                value_size_as_store_instruction(
                    variable.clone().value.datatype(context).unwrap().size(),
                    ir,
                );
            }
            GetVar => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if let Some(var) = context
                    .variables
                    .clone()
                    .get(self.parameters[0].word.as_ref().unwrap())
                {
                    ir_push(
                        IrInstruction {
                            kind: IrInstructionKind::PushVariable,
                            operand: IrInstructionValue::new().integer(var.id as i64),
                        },
                        ir,
                    );
                    value_size_as_load_instruction(var.value.clone().size(context), ir);
                } else if let Some(var) = context
                    .local_variables
                    .clone()
                    .get(self.parameters[0].word.as_ref().unwrap())
                {
                    ir_push(
                        IrInstruction {
                            kind: IrInstructionKind::PushVariable,
                            operand: IrInstructionValue::new().integer(var.id as i64),
                        },
                        ir,
                    );
                    value_size_as_load_instruction(var.value.clone().size(context), ir);
                } else {
                    return Err(LoispError::VariableNotFound(
                        self.parameters[0].token.clone(),
                    ));
                }
            }
            ChVar => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if let Some(var) = context
                    .variables
                    .get(self.parameters[0].word.as_ref().unwrap())
                {
                    let parameter1 = self.parameters[1].clone();
                    let variable_value = var.value.clone();
                    if parameter1.datatype(context).unwrap()
                        != variable_value.datatype(context).unwrap()
                    {
                        return Err(LoispError::MismatchedTypes(self.token.clone()));
                    }
                } else if let Some(var) = context
                    .local_variables
                    .get(self.parameters[0].word.as_ref().unwrap())
                {
                    let parameter1 = self.parameters[1].clone();
                    let variable_value = var.value.clone();
                    if parameter1.datatype(context).unwrap()
                        != variable_value.datatype(context).unwrap()
                    {
                        return Err(LoispError::MismatchedTypes(self.token.clone()));
                    }
                } else {
                    return Err(LoispError::VariableNotFound(
                        self.parameters[0].token.clone(),
                    ));
                }

                let var: LoispVariable;

                if let Some(v) = context
                    .variables
                    .get(self.parameters[0].word.as_ref().unwrap())
                {
                    var = v.clone();
                } else if let Some(v) = context
                    .local_variables
                    .get(self.parameters[0].word.as_ref().unwrap())
                {
                    var = v.clone();
                } else {
                    return Err(LoispError::VariableNotFound(self.parameters[0].token.clone()));
                }

                push_value(self.parameters.clone().last().unwrap().clone(), ir, context)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::PushVariable,
                        operand: IrInstructionValue::new().integer(var.id as i64),
                    },
                    ir,
                );
                value_size_as_store_instruction(
                    var.clone().value.datatype(context).unwrap().size(),
                    ir,
                );
            }
            While => {
                let loop_begin = ir.instructions.len() as i64;

                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                push_value(self.parameters[0].clone(), ir, context)?;

                let if_addr = ir.instructions.len();

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::If,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );

                {
                    let mut parameters = self.parameters.clone();
                    parameters.remove(0);
                    for p in parameters {
                        if p.is_instruction_return() {
                            if p.instruction_return.kind == LoispInstructionType::SetVar
                                || p.instruction_return.kind == LoispInstructionType::Alloc
                                || p.instruction_return.kind == LoispInstructionType::Macro
                                || p.instruction_return.kind == LoispInstructionType::DefFun
                            {
                                return Err(LoispError::NoDeclarationsInLoops(p.token));
                            }
                        }
                        push_value(p, ir, context)?;
                    }
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Jump,
                        operand: IrInstructionValue::new().integer(loop_begin),
                    },
                    ir,
                );

                let after_end = ir.instructions.len() as i64;
                ir.instructions[if_addr as usize].operand =
                    IrInstructionValue::new().integer(after_end + 1);

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Nop,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Nop,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Equal => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    || self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(
                        self.parameters[0].token.clone(),
                    ));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Equal,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            NotEqual => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    || self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(
                        self.parameters[0].token.clone(),
                    ));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::NotEqual,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            If => {
                if self.parameters.len() < 3 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 3 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                push_value(self.parameters[0].clone(), ir, context)?;

                let if_addr = ir.instructions.len() as i64;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::If,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );

                push_value(self.parameters[1].clone(), ir, context)?;

                let else_addr = ir.instructions.len() as i64;
                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Jump,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );

                let after_else = ir.instructions.len() as i64;
                ir.instructions[if_addr as usize].operand =
                    IrInstructionValue::new().integer(after_else + 1);

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Nop,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Nop,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );

                push_value(self.parameters[2].clone(), ir, context)?;

                let after_end = ir.instructions.len() as i64;
                ir.instructions[else_addr as usize].operand =
                    IrInstructionValue::new().integer(after_end + 1);

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Nop,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Nop,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Block => {
                self.push_parameters(ir, context, false)?;
            }
            Less => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    && self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Less,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Greater => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    && self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Greater,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            LessEqual => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    && self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::LessEqual,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            GreaterEqual => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    && self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::GreaterEqual,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            PtrTo => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if let Some(var) = context
                    .variables
                    .get(self.parameters[0].word.as_ref().unwrap())
                {
                    ir_push(
                        IrInstruction {
                            kind: IrInstructionKind::PushVariable,
                            operand: IrInstructionValue::new().integer(var.id as i64),
                        },
                        ir,
                    );
                } else {
                    return Err(LoispError::VariableNotFound(
                        self.parameters[0].token.clone(),
                    ));
                }
            }
            Load64 => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Pointer {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Load64,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Store64 => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Pointer
                    || self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Store64,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Load32 => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Pointer {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Load32,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Store32 => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Pointer
                    || self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Store32,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Load16 => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Pointer {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Load16,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Store16 => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Pointer
                    || self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Store16,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Load8 => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Pointer {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Load8,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Store8 => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Pointer
                    || self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Store8,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Alloc => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word
                    || self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if let Some(_) = context
                    .local_memories
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    return Err(LoispError::MemoryRedefinition(
                        self.parameters[0].token.clone(),
                    ));
                }

                if let Some(_) = context
                    .memories
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    return Err(LoispError::MemoryRedefinition(
                        self.parameters[0].token.clone(),
                    ));
                } else {
                    if self.parameters[1].is_instruction_return() {
                        return Err(LoispError::CantEvaluateAtCompileTime(self.token.clone()));
                    }

                    let memory = LoispMemory {
                        id: context.get_memory_count(),
                        alloc: self.parameters[1].integer.unwrap() as usize,
                    };

                    if context.inside_fun {
                        context
                            .local_memories
                            .insert(self.parameters[0].clone().word.unwrap(), memory);
                    } else {
                        context
                            .memories
                            .insert(self.parameters[0].clone().word.unwrap(), memory);
                    }
                }

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::AllocMemory,
                        operand: IrInstructionValue::new()
                            .integer(self.parameters[1].integer.unwrap()),
                    },
                    ir,
                );
            }
            GetMem => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if let Some(mem) = context
                    .memories
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    ir_push(
                        IrInstruction {
                            kind: IrInstructionKind::PushMemory,
                            operand: IrInstructionValue::new().integer(mem.id as i64),
                        },
                        ir,
                    );
                } else if let Some(mem) = context
                    .local_memories
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    ir_push(
                        IrInstruction {
                            kind: IrInstructionKind::PushMemory,
                            operand: IrInstructionValue::new().integer(mem.id as i64),
                        },
                        ir,
                    );
                } else {
                    return Err(LoispError::MemoryNotFound(self.parameters[0].token.clone()));
                }
            }
            CastPointer => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;
            }
            CastInt => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;
            }
            ShiftLeft => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    || self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::ShiftLeft,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            ShiftRight => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    || self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, false)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::ShiftRight,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            And => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    || self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::And,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Or => {
                if self.parameters.len() < 2 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 2 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer
                    || self.parameters[1].datatype(context).unwrap() != LoispDatatype::Integer
                {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Or,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Not => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Integer {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                self.push_parameters(ir, context, true)?;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Not,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );
            }
            Macro => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if let Some(_) = context
                    .macros
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    return Err(LoispError::MacroRedefinition(
                        self.parameters[0].token.clone(),
                    ));
                }

                let mut ops = IrProgram::new();
                {
                    let mut params = self.parameters.clone();
                    params.remove(0);
                    for p in params {
                        if p.is_instruction_return() {
                            if p.instruction_return.kind == LoispInstructionType::SetVar
                                || p.instruction_return.kind == LoispInstructionType::Alloc
                                || p.instruction_return.kind == LoispInstructionType::Macro
                                || p.instruction_return.kind == LoispInstructionType::DefFun
                            {
                                return Err(LoispError::NoDeclarationsInMacros(p.token));
                            }
                        }
                        push_value(p.clone(), &mut ops, context)?;
                    }
                }

                for i in &ops.instructions {
                    if i.kind == IrInstructionKind::Jump {
                        return Err(LoispError::NoJumpsInMacros(self.token.clone()));
                    }
                }

                let maccro = LoispMacro {
                    id: context.macros.len(),
                    program: ops.clone(),
                };

                context
                    .macros
                    .insert(self.parameters[0].clone().word.unwrap(), maccro);
            }
            Expand => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if let Some(mac) = context
                    .macros
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    for i in &mac.clone().program.instructions {
                        ir_push(i.clone(), ir);
                    }
                } else {
                    return Err(LoispError::MacroNotFound(self.parameters[0].token.clone()));
                }
            }
            Pop => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if let Some(var) = context
                    .variables
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    ir_push(
                        IrInstruction {
                            kind: IrInstructionKind::PushVariable,
                            operand: IrInstructionValue::new().integer(var.id as i64),
                        },
                        ir,
                    );
                    ir_push(
                        IrInstruction {
                            kind: IrInstructionKind::Store64,
                            operand: IrInstructionValue::new(),
                        },
                        ir,
                    );
                } else {
                    return Err(LoispError::VariableNotFound(
                        self.parameters[0].token.clone(),
                    ));
                }
            }
            Include => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters.len() > 1 {
                    return Err(LoispError::TooMuchParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::String {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if self.parameters[0].is_instruction_return() {
                    return Err(LoispError::CantEvaluateAtCompileTime(self.token.clone()));
                }

                fn exists(p: &str) -> bool {
                    return Path::new(p).exists();
                }

                let mut full_path = String::new();
                let mut encountered = false;
                let given_path = self.parameters[0].string.clone();

                for p in DEFAULT_SEARCH_PATHS {
                    let curp = format!("{}/{}", p, given_path);
                    if exists(curp.as_str()) {
                        full_path = curp.clone();
                        encountered = true;
                        break;
                    }
                }

                if !encountered {
                    full_path = given_path;
                }

                full_path = full_path.replace("//", "/").to_string();

                compile_file_into_existing_ir(full_path, ir, context)?;
            }
            DefFun => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                if let Some(_) = context
                    .functions
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    return Err(LoispError::FunctionRedefinition(
                        self.parameters[0].token.clone(),
                    ));
                }

                let defun_addr = ir.instructions.len() as i64;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Jump,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );

                let function_addr = ir.instructions.len() as i64;
                let mut function_type = LoispDatatype::Nothing;
                let previous_inside_func_state = context.inside_fun;
                context.inside_fun = true;
                {
                    let mut params = self.parameters.clone();
                    params.remove(0);
                    for p in params {
                        if p.is_instruction_return() {
                            if p.instruction_return.kind == LoispInstructionType::DefFun
                                || p.instruction_return.kind == LoispInstructionType::Macro
                            {
                                return Err(LoispError::NoDeclarationsInFunctions(p.token.clone()));
                            }
                        }
                        push_value(p.clone(), ir, context)?;
                    }

                    if let Some(i) = ir.instructions.last() {
                        function_type = i.get_loisp_datatype();
                    }
                }
                context.local_memories = HashMap::new();
                context.local_variables = HashMap::new();
                context.inside_fun = previous_inside_func_state;

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Return,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );

                let after_end = ir.instructions.len() as i64;
                ir.instructions[defun_addr as usize].operand =
                    IrInstructionValue::new().integer(after_end + 1);

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Nop,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );

                ir_push(
                    IrInstruction {
                        kind: IrInstructionKind::Nop,
                        operand: IrInstructionValue::new(),
                    },
                    ir,
                );

                let function = LoispFunction {
                    addr: function_addr as usize,
                    typ: function_type,
                };

                context
                    .functions
                    .insert(self.parameters[0].clone().word.unwrap(), function);
            }
            Call => {
                if self.parameters.len() < 1 {
                    return Err(LoispError::NotEnoughParameters(self.token.clone()));
                }

                if self.parameters[0].datatype(context).unwrap() != LoispDatatype::Word {
                    return Err(LoispError::MismatchedTypes(self.token.clone()));
                }

                {
                    let mut params = self.parameters.clone();
                    params.remove(0);
                    for p in params.iter().rev() {
                        push_value(p.clone(), ir, context)?;
                    }
                }

                if let Some(f) = context
                    .functions
                    .get(&self.parameters[0].clone().word.unwrap())
                {
                    ir_push(
                        IrInstruction {
                            kind: IrInstructionKind::Call,
                            operand: IrInstructionValue::new().integer(f.addr as i64),
                        },
                        ir,
                    );
                    ir_push(
                        IrInstruction {
                            kind: IrInstructionKind::Nop,
                            operand: IrInstructionValue::new().integer(f.addr as i64),
                        },
                        ir,
                    );
                } else {
                    return Err(LoispError::FunctionNotFound(
                        self.parameters[0].token.clone(),
                    ));
                }
            }
            Nop => {}
        }
        Ok(())
    }
}
