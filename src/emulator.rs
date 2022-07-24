use super::common::*;
use super::config::*;
use super::ir::*;

use std::collections::HashMap;
use std::io::*;

pub struct Emulator {
    pub args: Vec<String>,
    pub stack: Vec<i64>,
    pub ip: usize,

    pub string_size: usize,

    pub variables: HashMap<usize, usize>,
    pub variables_size: usize,

    pub memories: HashMap<usize, usize>,
    pub memories_size: usize,

    pub memory: Vec<u8>,
}

static NULL_PTR_PADDING: usize = 1;

static STRING_BUFFER_CAPACITY: usize = 640000; // should be enough for everyone
static STRING_BUFFER_START: usize = NULL_PTR_PADDING;

static VARIABLE_BUFFER_CAPACITY: usize = 640000; // should be enough for everyone
static VARIABLE_BUFFER_START: usize = STRING_BUFFER_START + STRING_BUFFER_CAPACITY;

static MEMORY_BUFFER_CAPACITY: usize = 640000; // should be enough for everyone
static MEMORY_BUFFER_START: usize = VARIABLE_BUFFER_START + VARIABLE_BUFFER_CAPACITY;

static X86_64_MEMORY_CAPACITY: usize =
    NULL_PTR_PADDING + STRING_BUFFER_CAPACITY + VARIABLE_BUFFER_CAPACITY + MEMORY_BUFFER_CAPACITY;

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            args: vec![],
            stack: vec![],
            ip: 0,

            string_size: STRING_BUFFER_START,

            variables: HashMap::new(),
            variables_size: VARIABLE_BUFFER_START,

            memories: HashMap::new(),
            memories_size: MEMORY_BUFFER_START,

            memory: vec![0; X86_64_MEMORY_CAPACITY],
        }
    }
}

pub fn emulate_program(ir: IrProgram, emulator: &mut Emulator) {
    while emulator.ip < ir.instructions.len() {
        let op = ir.instructions[emulator.ip].clone();
        match op.kind {
            IrInstructionKind::Print => {
                if let Some(a) = emulator.stack.pop() {
                    println!("{}", a);
                    emulator.ip += 1;
                }
            }
            IrInstructionKind::PushInteger => {
                emulator.stack.push(op.operand.integer);
                emulator.ip += 1;
            }
            IrInstructionKind::Plus => {
                let a;
                let b;

                if let Some(v) = emulator.stack.pop() {
                    a = v;
                } else {
                    panic!("stack underflow")
                }

                if let Some(v) = emulator.stack.pop() {
                    b = v;
                } else {
                    panic!("stack underflow")
                }

                emulator.stack.push(a + b);
                emulator.ip += 1;
            }
            IrInstructionKind::Minus => todo!("Minus"),
            IrInstructionKind::Multiplication => {
                let a;
                let b;

                if let Some(v) = emulator.stack.pop() {
                    a = v;
                } else {
                    panic!("stack underflow")
                }

                if let Some(v) = emulator.stack.pop() {
                    b = v;
                } else {
                    panic!("stack underflow")
                }

                emulator.stack.push(a * b);
                emulator.ip += 1;
            }
            IrInstructionKind::Division => todo!("Division"),
            IrInstructionKind::Mod => {
                let a;
                let b;

                if let Some(v) = emulator.stack.pop() {
                    a = v;
                } else {
                    panic!("stack underflow")
                }

                if let Some(v) = emulator.stack.pop() {
                    b = v;
                } else {
                    panic!("stack underflow")
                }

                emulator.stack.push(a % b);
                emulator.ip += 1;
            }
            IrInstructionKind::Syscall => {
                let syscall_number;
                if let Some(id) = emulator.stack.pop() {
                    syscall_number = id;
                } else {
                    panic!("stack underflow");
                }

                match syscall_number {
                    1 => {
                        let fd;
                        let buf;
                        let count;

                        if let Some(d) = emulator.stack.pop() {
                            fd = d;
                        } else {
                            panic!("stack underflow");
                        }

                        if let Some(b) = emulator.stack.pop() {
                            buf = b;
                        } else {
                            panic!("stack underflow");
                        }

                        if let Some(c) = emulator.stack.pop() {
                            count = c;
                        } else {
                            panic!("stack underflow");
                        }

                        let mut buffer = String::new();
                        let mut i = 0;
                        while i < count {
                            buffer.push(emulator.memory[(buf + i) as usize] as char);
                            i += 1;
                        }

                        match fd {
                            1 => print!("{}", buffer),
                            2 => eprint!("{}", buffer),
                            _ => panic!("unknown file descriptor"),
                        }
                    }
                    _ => panic!("unsupported syscall {}", syscall_number),
                }
                emulator.ip += 1;
            }
            IrInstructionKind::AllocVariable => {
                emulator
                    .variables
                    .insert(emulator.variables.len(), emulator.variables_size);
                emulator.variables_size += op.operand.integer as usize;
                emulator.ip += 1;
            }
            IrInstructionKind::Load8 => {
                let addr;
                if let Some(a) = emulator.stack.pop() {
                    addr = a;
                } else {
                    panic!("stack underflow");
                }

                emulator.stack.push(emulator.memory[addr as usize] as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::Store8 => {
                let addr;
                let value;

                if let Some(a) = emulator.stack.pop() {
                    addr = a;
                } else {
                    panic!("stack underflow");
                }

                if let Some(v) = emulator.stack.pop() {
                    value = v;
                } else {
                    panic!("stack underflow");
                }

                emulator.memory[addr as usize] = value as u8;
                emulator.ip += 1;
            }
            IrInstructionKind::Load16 => todo!("Load16"),
            IrInstructionKind::Store16 => todo!("Store16"),
            IrInstructionKind::Load32 => todo!("Load32"),
            IrInstructionKind::Store32 => todo!("Store32"),
            IrInstructionKind::Load64 => {
                let addr;
                if let Some(a) = emulator.stack.pop() {
                    addr = a;
                } else {
                    panic!("stack underflow");
                }

                let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                for i in 0..8 {
                    bytes[i] = emulator.memory[(addr as usize) + i];
                }
                emulator.stack.push(i64::from_le_bytes(bytes));
                emulator.ip += 1;
            }
            IrInstructionKind::Store64 => {
                let mut addr;
                let value;

                if let Some(a) = emulator.stack.pop() {
                    addr = a;
                } else {
                    panic!("stack underflow");
                }

                if let Some(v) = emulator.stack.pop() {
                    value = v;
                } else {
                    panic!("stack underflow");
                }

                let bytes = value.to_le_bytes();
                for b in bytes {
                    emulator.memory[addr as usize] = b;
                    addr += 1;
                }

                emulator.ip += 1;
            }
            IrInstructionKind::PushVariable => {
                if let Some(addr) = emulator.variables.get(&(op.operand.integer as usize)) {
                    emulator.stack.push(*addr as i64);
                }
                emulator.ip += 1;
            }
            IrInstructionKind::Jump => {
                emulator.ip = op.operand.integer as usize;
            }
            IrInstructionKind::Nop => emulator.ip += 1,
            IrInstructionKind::If => {
                if let Some(a) = emulator.stack.pop() {
                    if a != 0 {
                        emulator.ip += 1;
                    } else {
                        emulator.ip = op.operand.integer as usize;
                    }
                } else {
                    panic!("stack underflow");
                }
            }
            IrInstructionKind::Equal => {
                let a;
                let b;

                if let Some(v) = emulator.stack.pop() {
                    a = v;
                } else {
                    panic!("stack underflow")
                }

                if let Some(v) = emulator.stack.pop() {
                    b = v;
                } else {
                    panic!("stack underflow")
                }

                emulator.stack.push((b == a) as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::NotEqual => todo!("NotEqual"),
            IrInstructionKind::Less => {
                let a;
                let b;

                if let Some(v) = emulator.stack.pop() {
                    a = v;
                } else {
                    panic!("stack underflow")
                }

                if let Some(v) = emulator.stack.pop() {
                    b = v;
                } else {
                    panic!("stack underflow")
                }

                emulator.stack.push((a < b) as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::Greater => todo!("Greater"),
            IrInstructionKind::LessEqual => todo!("LessEqual"),
            IrInstructionKind::GreaterEqual => todo!("GreaterEqual"),
            IrInstructionKind::AllocMemory => {
                emulator
                    .memories
                    .insert(emulator.memories.len(), emulator.memories_size);
                emulator.memories_size += op.operand.integer as usize;
                emulator.ip += 1;
            }
            IrInstructionKind::PushMemory => {
                if let Some(addr) = emulator.memories.get(&(op.operand.integer as usize)) {
                    emulator.stack.push(*addr as i64);
                }
                emulator.ip += 1;
            }
            IrInstructionKind::ShiftLeft => todo!("ShiftLeft"),
            IrInstructionKind::ShiftRight => todo!("ShiftRight"),
            IrInstructionKind::Or => todo!("Or"),
            IrInstructionKind::And => todo!("And"),
            IrInstructionKind::Not => todo!("Not"),
            IrInstructionKind::PushString => {
                let addr = emulator.string_size;
                let string = escape_string(op.operand.string);
                for c in string.chars() {
                    emulator.memory[emulator.string_size] = c as u8;
                    emulator.string_size += 1;
                }
                emulator.stack.push(addr as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::Call => todo!("Call"),
            IrInstructionKind::Return => todo!("Return"),
        }
    }
}

pub fn emulate_file(config: Config) -> Result<()> {
    let ir = compile_file_into_ir(config.clone().input)?;
    let mut emulator = Emulator::new();
    emulator.args = config.run.args;
    emulate_program(ir, &mut emulator);
    Ok(())
}
