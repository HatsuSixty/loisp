use super::types::*;
use super::ir::*;
use super::parser::*;

#[derive(Debug)]
pub enum LoispError {
    NotEnoughParameters(String),
    TooMuchParameters(String),
    CantAcceptNothing(String),
    MismatchedTypes(String),
    ParserError(ParserError),
    Unknown
}

impl From<std::io::Error> for LoispError {
    fn from(_: std::io::Error) -> Self {
        Self::Unknown
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
    Nop
}

impl LoispInstructionType {
    pub fn return_type(&self) -> LoispDatatype {
        use LoispDatatype::*;
        match self {
            Self::Print => Nothing,
            Self::Nop   => Nothing,
            Self::Plus  => Integer
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
    pub fn new() -> LoispValue {
        LoispValue {
            integer: None,
            string: String::new(),
            instruction_return: LoispInstruction::new()
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
    pub parameters: Vec<LoispValue>
}

impl LoispInstruction {
    pub fn new() -> LoispInstruction {
        LoispInstruction {
            kind: LoispInstructionType::Nop,
            parameters: vec![]
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
                    return Err(NotEnoughParameters("print".to_string()))
                }

                if self.parameters.len() > 1 {
                    return Err(TooMuchParameters("print".to_string()))
                }

                if self.parameters[0].datatype().unwrap() == LoispDatatype::Nothing {
                    return Err(CantAcceptNothing("print".to_string()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Print, operand: IrInstructionValue::new()});
            }
            Plus => {
                if self.parameters.len() < 2 {
                    return Err(NotEnoughParameters("+".to_string()))
                }

                if self.parameters.len() > 2 {
                    return Err(TooMuchParameters("+".to_string()))
                }

                if !(self.parameters[0].datatype().unwrap() == LoispDatatype::Integer
                     && self.parameters[1].datatype().unwrap() == LoispDatatype::Integer) {
                    return Err(MismatchedTypes("+".to_string()))
                }

                ir.push(IrInstruction {kind: IrInstructionKind::Plus, operand: IrInstructionValue::new()});
            }
            Nop => {}
        }
        Ok(())
    }
}
