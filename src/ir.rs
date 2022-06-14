use std::fs::File;
use std::fs::OpenOptions;
use std::io::*;

#[derive(Debug)]
pub enum IrInstructionKind {
    PushInteger,
    Plus,
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
    pub fn to_nasm_linux_x86_64_assembly(&self, f: &mut File) -> Result<()> {
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
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(output)
            .expect("Could not open file {}");

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
