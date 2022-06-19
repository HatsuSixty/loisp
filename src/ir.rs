use super::parser::*;
use super::lexer::*;
use super::common::*;
use super::config::*;

use std::fs;
use std::io::*;

pub fn syscall_number_as_register(n: i64) -> String {
    match n {
        0 => "rax".to_string(),
        1 => "rdi".to_string(),
        2 => "rsi".to_string(),
        3 => "r10".to_string(),
        4 => "r8".to_string(),
        5 => "r9".to_string(),
        _ => "invalid".to_string()
    }
}

#[derive(Debug)]
pub enum IrInstructionKind {
    PushInteger,
    Plus,
    Minus,
    Multiplication,
    Division,
    Mod,
    Syscall,
    Print
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct IrInstructionValue {
    string: String,
    integer: i64
}

impl IrInstructionValue {
    pub fn new() -> IrInstructionValue {
        IrInstructionValue {
            string: String::new(),
            integer: 0
        }
    }

    pub fn integer(&mut self, integer: i64) -> Self {
        self.integer = integer;
        self.clone()
    }
}

pub struct IrInstruction {
    pub kind: IrInstructionKind,
    pub operand: IrInstructionValue
}

impl IrInstruction {
    pub fn to_nasm_linux_x86_64_assembly(&self, f: &mut fs::File) -> Result<()> {
        use IrInstructionKind::*;

        match self.kind {
            PushInteger => {
                writeln!(f, "mov rax, {}", self.operand.integer)?;
                writeln!(f, "push rax")?;
            }
            Print => {
                writeln!(f, "pop rdi")?;
                writeln!(f, "call print")?;
            }
            Plus => {
                writeln!(f, "pop rax")?;
                writeln!(f, "pop rbx")?;
                writeln!(f, "add rax, rbx")?;
                writeln!(f, "push rax")?;
            }
            Minus => {
                writeln!(f, "pop rax")?;
                writeln!(f, "pop rbx")?;
                writeln!(f, "sub rax, rbx")?;
                writeln!(f, "push rax")?;
            }
            Multiplication => {
                writeln!(f, "pop rax")?;
                writeln!(f, "pop rbx")?;
                writeln!(f, "mul rbx")?;
                writeln!(f, "push rax")?;
            }
            Division => {
                writeln!(f, "pop rax")?;
                writeln!(f, "pop rbx")?;
                writeln!(f, "div rbx")?;
                writeln!(f, "push rax")?;
            }
            Mod => {
                writeln!(f, "xor rdx, rdx")?;
                writeln!(f, "pop rax")?;
                writeln!(f, "pop rbx")?;
                writeln!(f, "div rbx")?;
                writeln!(f, "push rdx")?;
            }
            Syscall => {
                for i in 0..self.operand.integer {
                    writeln!(f, "pop {}", syscall_number_as_register(i))?;
                }
                writeln!(f, "syscall")?;
                writeln!(f, "push rax")?;
            }
        }

        Ok(())
    }
}

pub struct IrProgram {
    pub instructions: Vec<IrInstruction>
}

impl IrProgram {
    pub fn new() -> IrProgram {
        IrProgram {
            instructions: vec![]
        }
    }

    pub fn push(&mut self, i: IrInstruction) {
        self.instructions.push(i)
    }

    pub fn to_nasm_linux_x86_64_assembly(&self, output: String) -> Result<()> {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(output)
            .expect("Could not open file {}");

        writeln!(f, "BITS 64")?;
        writeln!(f, "print:")?;
        writeln!(f, "mov r9, -3689348814741910323")?;
        writeln!(f, "sub rsp, 40")?;
        writeln!(f, "mov BYTE [rsp+31], 10")?;
        writeln!(f, "lea rcx, [rsp+30]")?;
        writeln!(f, ".L2:")?;
        writeln!(f, "mov rax, rdi")?;
        writeln!(f, "lea r8, [rsp+32]")?;
        writeln!(f, "mul r9")?;
        writeln!(f, "mov rax, rdi")?;
        writeln!(f, "sub r8, rcx")?;
        writeln!(f, "shr rdx, 3")?;
        writeln!(f, "lea rsi, [rdx+rdx*4]")?;
        writeln!(f, "add rsi, rsi")?;
        writeln!(f, "sub rax, rsi")?;
        writeln!(f, "add eax, 48")?;
        writeln!(f, "mov BYTE [rcx], al")?;
        writeln!(f, "mov rax, rdi")?;
        writeln!(f, "mov rdi, rdx")?;
        writeln!(f, "mov rdx, rcx")?;
        writeln!(f, "sub rcx, 1")?;
        writeln!(f, "cmp rax, 9")?;
        writeln!(f, "ja  .L2")?;
        writeln!(f, "lea rax, [rsp+32]")?;
        writeln!(f, "mov edi, 1")?;
        writeln!(f, "sub rdx, rax")?;
        writeln!(f, "xor eax, eax")?;
        writeln!(f, "lea rsi, [rsp+32+rdx]")?;
        writeln!(f, "mov rdx, r8")?;
        writeln!(f, "mov rax, 1")?;
        writeln!(f, "syscall")?;
        writeln!(f, "add rsp, 40")?;
        writeln!(f, "ret")?;
        writeln!(f, "global _start")?;
        writeln!(f, "_start:")?;

        for i in &self.instructions {
            writeln!(f, ";; -- {:?} --", i.kind)?;
            i.to_nasm_linux_x86_64_assembly(&mut f)?;
        }

        writeln!(f, "mov rax, 60")?;
        writeln!(f, "mov rdi, 0")?;
        writeln!(f, "syscall")?;

        Ok(())
    }
}

pub fn compile_file_into_ir(f: String) -> Result<IrProgram> {
    let source = fs::read_to_string(f.as_str())?;
    let lexer = Lexer::from_chars(source.chars(), f);

    let result = construct_instructions_from_tokens(&mut lexer.peekable());
    if let Err(error) = result {
        eprintln!("{}", error);
        std::process::exit(1);
    }
    let instructions = result.unwrap();

    let mut ir = IrProgram::new();
    for i in instructions {
        let result = i.to_ir(&mut ir);
        if let Err(error) = result {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }

    Ok(ir)
}

pub fn compile_file_into_assembly(i: &str, o: &str) -> Result<()> {
    let ir = compile_file_into_ir(i.to_string())?;
    ir.to_nasm_linux_x86_64_assembly(o.to_string())?;
    Ok(())
}

pub fn compile_file_into_executable(config: Config) -> Result<()> {
    let output_assembly = format!("{}.asm", config.output);
    let output_object = format!("{}.o", config.output);

    compile_file_into_assembly(config.input.as_str(), output_assembly.as_str())?;

    let assembler_command =
        format!("yasm -gdwarf2 -felf64 {} -o {}", output_assembly, output_object);
    let linker_command =
        format!("ld -o {} {}", config.output, output_object);

    run_command_with_info(assembler_command, config.clone())?;
    run_command_with_info(linker_command, config.clone())?;
    if config.run {
        run_command_with_info(format!("./{}", config.output), config)?;
    }

    Ok(())
}
