use super::ir::*;
use super::config::*;
use super::common::*;

use std::io::*;

pub struct Emulator {
    pub args: Vec<String>,
    pub stack: Vec<i64>,
    pub ip: usize,

    pub string_size: usize,

    pub memory: Vec<u8>,
}

static STRING_BUFFER_START: usize = 0;
static STRING_BUFFER_CAPACITY: usize = 640000; // should be enough for everyone

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            args: vec![],
            stack: vec![],
            ip: 0,

            string_size: STRING_BUFFER_START,

            memory: vec![0; STRING_BUFFER_CAPACITY],
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
            IrInstructionKind::Plus => todo!("Plus"),
            IrInstructionKind::Minus => todo!("Minus"),
            IrInstructionKind::Multiplication => todo!("Multiplication"),
            IrInstructionKind::Division => todo!("Division"),
            IrInstructionKind::Mod => todo!("Mod"),
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
            IrInstructionKind::AllocVariable => todo!("AllocVariable"),
            IrInstructionKind::Load8 => todo!("Load8"),
            IrInstructionKind::Store8 => todo!("Store8"),
            IrInstructionKind::Load16 => todo!("Load16"),
            IrInstructionKind::Store16 => todo!("Store16"),
            IrInstructionKind::Load32 => todo!("Load32"),
            IrInstructionKind::Store32 => todo!("Store32"),
            IrInstructionKind::Load64 => todo!("Load64"),
            IrInstructionKind::Store64 => todo!("Store64"),
            IrInstructionKind::PushVariable => todo!("PushVariable"),
            IrInstructionKind::Jump => todo!("Jump"),
            IrInstructionKind::Nop => todo!("Nop"),
            IrInstructionKind::If => todo!("If"),
            IrInstructionKind::Equal => todo!("Equal"),
            IrInstructionKind::NotEqual => todo!("NotEqual"),
            IrInstructionKind::Less => todo!("Less"),
            IrInstructionKind::Greater => todo!("Greater"),
            IrInstructionKind::LessEqual => todo!("LessEqual"),
            IrInstructionKind::GreaterEqual => todo!("GreaterEqual"),
            IrInstructionKind::AllocMemory => todo!("AllocMemory"),
            IrInstructionKind::PushMemory => todo!("PushMemory"),
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
