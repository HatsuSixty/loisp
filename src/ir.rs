use super::common::*;
use super::config::*;
use super::instructions::*;
use super::lexer::*;
use super::parser::*;
use super::print_info;

use std::fs;
use std::io::*;

static IR_ASSERT_ENABLED: bool = false;

macro_rules! assert_if_enabled {
    ($($arg:tt)*) => {{
        if IR_ASSERT_ENABLED {
            assert!($($arg)*);
        }
    }};
}

pub fn syscall_number_as_register(n: i64) -> String {
    match n {
        0 => "rax".to_string(),
        1 => "rdi".to_string(),
        2 => "rsi".to_string(),
        3 => "r10".to_string(),
        4 => "r8".to_string(),
        5 => "r9".to_string(),
        _ => "invalid".to_string(),
    }
}

#[derive(Debug)]
pub enum IrInstructionKind {
    Print,
    PushInteger,
    Plus,
    Minus,
    Multiplication,
    Division,
    Mod,
    Syscall,
    AllocMemory,
    Load8,
    Store8,
    Load16,
    Store16,
    Load32,
    Store32,
    Load64,
    Store64,
    PushMemory,
    Jump,
    Nop,
    If,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct IrInstructionValue {
    string: String,
    integer: i64,
}

impl IrInstructionValue {
    pub fn new() -> IrInstructionValue {
        IrInstructionValue {
            string: String::new(),
            integer: 0,
        }
    }

    pub fn integer(&mut self, integer: i64) -> Self {
        self.integer = integer;
        self.clone()
    }
}

pub struct IrMemory {
    pub ident: usize,
    pub alloc: usize,
}

pub struct IrContext {
    pub memories: Vec<IrMemory>,
    pub label_count: i64,
}

impl IrContext {
    pub fn new() -> IrContext {
        IrContext {
            memories: vec![],
            label_count: 1,
        }
    }
}

pub struct IrInstruction {
    pub kind: IrInstructionKind,
    pub operand: IrInstructionValue,
}

impl IrInstruction {
    pub fn to_intel_linux_x86_64_assembly(
        &self,
        f: &mut fs::File,
        context: &mut IrContext,
    ) -> Result<()> {
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
            AllocMemory => {
                let memory = IrMemory {
                    ident: context.memories.len(),
                    alloc: self.operand.integer as usize,
                };
                context.memories.push(memory);
            }
            Load8 => {
                writeln!(f, "pop rax")?;
                writeln!(f, "xor rbx, rbx")?;
                writeln!(f, "mov bl, [rax]")?;
                writeln!(f, "push rbx")?;
            }
            Store8 => {
                writeln!(f, "pop rax")?;
                writeln!(f, "pop rbx")?;
                writeln!(f, "mov [rax], bl")?;
            }
            Load16 => {
                writeln!(f, "pop rax")?;
                writeln!(f, "xor rbx, rbx")?;
                writeln!(f, "mov bx, [rax]")?;
                writeln!(f, "push rbx")?;
            }
            Store16 => {
                writeln!(f, "pop rax")?;
                writeln!(f, "pop rbx")?;
                writeln!(f, "mov [rax], bx")?;
            }
            Load32 => {
                writeln!(f, "pop rax")?;
                writeln!(f, "xor rbx, rbx")?;
                writeln!(f, "mov ebx, [rax]")?;
                writeln!(f, "push rbx")?;
            }
            Store32 => {
                writeln!(f, "pop rax")?;
                writeln!(f, "pop rbx")?;
                writeln!(f, "mov [rax], ebx")?;
            }
            Load64 => {
                writeln!(f, "pop rax")?;
                writeln!(f, "xor rbx, rbx")?;
                writeln!(f, "mov rbx, [rax]")?;
                writeln!(f, "push rbx")?;
            }
            Store64 => {
                writeln!(f, "pop rax")?;
                writeln!(f, "pop rbx")?;
                writeln!(f, "mov [rax], rbx")?;
            }
            PushMemory => {
                for m in &context.memories {
                    if m.ident == (self.operand.integer as usize) {
                        writeln!(f, "push mem_{}", self.operand.integer)?;
                        break;
                    }
                }
            }
            Jump => {
                assert_if_enabled!(
                    self.operand.integer <= context.label_count,
                    "Label does not exist"
                );
                writeln!(f, "jmp addr_{}", self.operand.integer)?;
            }
            If => {
                assert_if_enabled!(
                    self.operand.integer <= context.label_count,
                    "Label does not exist"
                );
                writeln!(f, "pop rax")?;
                writeln!(f, "test rax, rax")?;
                writeln!(f, "jz addr_{}", self.operand.integer)?;
            }
            Nop => {}
        }

        Ok(())
    }
}

pub struct IrProgram {
    pub instructions: Vec<IrInstruction>,
}

impl IrProgram {
    pub fn new() -> IrProgram {
        IrProgram {
            instructions: vec![],
        }
    }

    pub fn push(&mut self, i: IrInstruction) {
        self.instructions.push(i)
    }

    pub fn to_fasm_linux_x86_64_assembly(
        &self,
        output: String,
        config: Config,
        context: &mut IrContext,
    ) -> Result<()> {
        if !config.silent {
            print_info!("INFO", "Generating `{}`", output);
        }

        let mut f = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(output)
            .expect("Could not open file {}");

        writeln!(f, "format ELF64 executable 3")?;
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
        writeln!(f, "entry start")?;
        writeln!(f, "start:")?;

        for (k, i) in self.instructions.iter().enumerate() {
            writeln!(f, "addr_{}:", k)?;
            context.label_count += 1;
            writeln!(f, ";; -- {:?} --", i.kind)?;
            i.to_intel_linux_x86_64_assembly(&mut f, context)?;
        }

        writeln!(f, "mov rax, 60")?;
        writeln!(f, "mov rdi, 0")?;
        writeln!(f, "syscall")?;
        writeln!(f, "segment readable writable")?;

        for m in &context.memories {
            writeln!(f, "mem_{}: rb {}", m.ident, m.alloc)?;
        }

        Ok(())
    }
}

pub fn compile_file_into_ir(f: String) -> Result<IrProgram> {
    let source = fs::read_to_string(f.as_str())?;
    let lexer = Lexer::from_chars(source.chars(), f);
    let mut context = LoispContext::new();

    let result = construct_instructions_from_tokens(&mut lexer.peekable());
    if let Err(error) = result {
        eprintln!("{}", error);
        std::process::exit(1);
    }
    let instructions = result.unwrap();

    let mut ir = IrProgram::new();
    for i in instructions {
        let result = i.to_ir(&mut ir, &mut context);
        if let Err(error) = result {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }

    Ok(ir)
}

pub fn compile_file_into_assembly(i: &str, o: &str, config: Config) -> Result<()> {
    let mut context = IrContext::new();
    let ir = compile_file_into_ir(i.to_string())?;
    ir.to_fasm_linux_x86_64_assembly(o.to_string(), config.clone(), &mut context)?;
    Ok(())
}

pub fn compile_file_into_executable(config: Config) -> Result<()> {
    let config_output: String;
    {
        let c = config.clone();
        if let Some(o) = c.output {
            config_output = o;
        } else {
            config_output = file_name_without_extension(c.input);
        }
    }
    let output_assembly = format!("{}.asm", config_output);
    let output_executable = format!("{}.tmp", config_output);

    compile_file_into_assembly(
        config.input.as_str(),
        output_assembly.as_str(),
        config.clone(),
    )?;

    let assembler_command = format!("fasm -m 524288 {} {}", output_assembly, output_executable);
    let chmod_command = format!("chmod +x {}", output_executable);
    let rename_command = format!("mv {} {}", output_executable, config_output);

    run_command_with_info(assembler_command, config.clone())?;
    run_command_with_info(chmod_command, config.clone())?;
    run_command_with_info(rename_command, config.clone())?;

    {
        let mut c = config.clone();
        c.piped = false;
        if config.run.run {
            let mut command = format!("./{}", config_output);
            for a in config.run.args {
                command = format!("{} {}", command, a);
            }
            run_command_with_info(command, c)?;
        }
    }

    Ok(())
}
