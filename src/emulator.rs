use super::common::*;
use super::config::*;
use super::ir::*;

use std::collections::HashMap;
use std::env::vars;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::*;
use std::process::*;

pub struct Stream {
    pub stdout: Option<Stdout>,
    pub stderr: Option<Stderr>,
    pub stdin: Option<Stdin>,
    pub file: Option<File>,
}

impl Stream {
    pub fn new() -> Stream {
        Stream {
            stdout: None,
            stderr: None,
            stdin: None,
            file: None,
        }
    }

    pub fn write(&self, s: String) -> Result<()> {
        if !self.stdout.is_none() {
            write!(self.stdout.as_ref().unwrap(), "{}", s)?;
            self.stdout.as_ref().unwrap().flush()?;
        } else if !self.stderr.is_none() {
            write!(self.stderr.as_ref().unwrap(), "{}", s)?;
            self.stderr.as_ref().unwrap().flush()?;
        } else if !self.file.is_none() {
            write!(self.file.as_ref().unwrap(), "{}", s)?;
            self.file.as_ref().unwrap().flush()?;
        } else if !self.stdin.is_none() {
            return Err(Error::new(ErrorKind::Other, "EBADFD"));
        }

        Ok(())
    }

    pub fn read(&self) -> Result<String> {
        let mut result = String::new();

        if !self.stdout.is_none() {
            stdin().read_line(&mut result)?;
        } else if !self.stderr.is_none() {
            stdin().read_line(&mut result)?;
        } else if !self.stdin.is_none() {
            stdin().read_line(&mut result)?;
        } else if !self.file.is_none() {
            let mut bytes: Vec<u8> = vec![];
            self.file.as_ref().unwrap().read_to_end(&mut bytes)?;
            result = match String::from_utf8(bytes.to_vec()) {
                Ok(s) => s,
                Err(_) => String::new(),
            }
        }

        Ok(result)
    }
}

pub struct Emulator {
    pub args: Vec<String>,
    pub stack: Vec<i64>,
    pub ip: usize,

    pub string_size: usize,

    pub variables: HashMap<usize, usize>,
    pub variables_size: usize,

    pub memories: HashMap<usize, usize>,
    pub memories_size: usize,

    pub ret_stack: Vec<usize>,

    pub fds: HashMap<usize, Stream>,

    pub memory: Vec<u8>,
}

static NULL_PTR_PADDING: usize = 1;

static STRING_BUFFER_CAPACITY: usize = 640000; // should be enough for everyone
static STRING_BUFFER_START: usize = NULL_PTR_PADDING;

static VARIABLE_BUFFER_CAPACITY: usize = 640000; // should be enough for everyone
static VARIABLE_BUFFER_START: usize = STRING_BUFFER_START + STRING_BUFFER_CAPACITY;

static MEMORY_BUFFER_CAPACITY: usize = 640000; // should be enough for everyone
static MEMORY_BUFFER_START: usize = VARIABLE_BUFFER_START + VARIABLE_BUFFER_CAPACITY;

static ARGS_BUFFER_CAPACITY: usize = 640000; // should be enough for everyone
static ARGS_BUFFER_START: usize = MEMORY_BUFFER_START + MEMORY_BUFFER_CAPACITY;

static VARS_BUFFER_CAPACITY: usize = 640000; // should be enough for everyone
static VARS_BUFFER_START: usize = ARGS_BUFFER_START + ARGS_BUFFER_CAPACITY;

static X86_64_MEMORY_CAPACITY: usize = NULL_PTR_PADDING
    + STRING_BUFFER_CAPACITY
    + VARIABLE_BUFFER_CAPACITY
    + MEMORY_BUFFER_CAPACITY
    + ARGS_BUFFER_CAPACITY
    + VARS_BUFFER_CAPACITY;

impl Emulator {
    pub fn new() -> Emulator {
        let mut ctx = Emulator {
            args: vec![],
            stack: vec![],
            ip: 0,

            string_size: STRING_BUFFER_START,

            variables: HashMap::new(),
            variables_size: VARIABLE_BUFFER_START,

            memories: HashMap::new(),
            memories_size: MEMORY_BUFFER_START,

            ret_stack: vec![],

            fds: HashMap::new(),

            memory: vec![0; X86_64_MEMORY_CAPACITY],
        };

        let mut fd0 = Stream::new();
        let mut fd1 = Stream::new();
        let mut fd2 = Stream::new();
        fd1.stdout = Some(stdout());
        fd2.stderr = Some(stderr());
        fd0.stdin = Some(stdin());

        ctx.fds.insert(0, fd0);
        ctx.fds.insert(1, fd1);
        ctx.fds.insert(2, fd2);

        ctx
    }

    pub fn init(&mut self, ir: IrProgram) {
        for i in ir.instructions {
            match i.kind {
                IrInstructionKind::AllocVariable => {
                    self.variables
                        .insert(self.variables.len(), self.variables_size);
                    self.variables_size += i.operand.integer as usize;
                }
                IrInstructionKind::AllocMemory => {
                    self.memories
                        .insert(self.memories.len(), self.memories_size);
                    self.memories_size += i.operand.integer as usize;
                }
                IrInstructionKind::PushString => {
                    let string = escape_string(i.operand.string);
                    for c in string.as_bytes() {
                        self.memory[self.string_size] = *c;
                        self.string_size += 1;
                    }
                    self.memory[self.string_size] = 0;
                    self.string_size += 1;
                }
                _ => {}
            }
        }
    }

    pub fn find_string_in_memory(&self, string: String) -> Option<usize> {
        let mut pos_search = 0;
        let mut pos_text = 0;
        let len_search = string.len();
        let len_text = STRING_BUFFER_CAPACITY;

        if len_search == 0 {
            return Some(0);
        }

        while pos_text < len_text - len_search {
            if self.memory[pos_text] == string.as_bytes()[pos_search] {
                pos_search += 1;
                if pos_search == len_search {
                    return Some(pos_text - (len_search - 1));
                }
            } else {
                pos_text -= pos_search;
                pos_search = 0;
            }

            pos_text += 1;
        }

        return None;
    }
}

pub fn emulate_program(ir: IrProgram, emulator: &mut Emulator) {
    let argv;
    {
        let mut ptrs: Vec<u64> = vec![];
        let mut i: usize = 0;
        for arg in &emulator.args {
            ptrs.push((ARGS_BUFFER_START + i) as u64);
            for c in arg.as_bytes() {
                emulator.memory[ARGS_BUFFER_START + i] = *c as u8;
                i += 1;
            }
            emulator.memory[ARGS_BUFFER_START + i] = 0;
            i += 1;
        }

        argv = (ARGS_BUFFER_START + i) as i64;
        for ptr in ptrs {
            for b in ptr.to_le_bytes() {
                emulator.memory[ARGS_BUFFER_START + i] = b;
                i += 1;
            }
        }
    }

    let envp;
    {
        let mut ptrs: Vec<u64> = vec![];
        let mut i: usize = 0;
        for (key, value) in vars() {
            let env = format!("{}={}", key, value);

            ptrs.push((VARS_BUFFER_START + i) as u64);
            for b in env.clone().as_bytes() {
                emulator.memory[VARS_BUFFER_START + i] = *b;
                i += 1;
            }
            emulator.memory[VARS_BUFFER_START + i] = 0;
            i += 1;
        }
        ptrs.push(0);

        envp = (VARS_BUFFER_START + i) as i64;
        for p in ptrs {
            for b in p.to_le_bytes() {
                emulator.memory[VARS_BUFFER_START + i] = b;
                i += 1;
            }
        }
    }

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
            IrInstructionKind::Minus => {
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

                emulator.stack.push(a - b);
                emulator.ip += 1;
            }
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
            IrInstructionKind::Division => {
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

                emulator.stack.push((a / b) as i64);
                emulator.ip += 1;
            }
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
                    0 => {
                        // SYS_read
                        let fd;
                        let mut buf;
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

                        let buffer;
                        if let Some(stream) = emulator.fds.get(&(fd as usize)) {
                            buffer = match stream.read() {
                                Ok(s) => s,
                                Err(_) => String::new(),
                            };
                        } else {
                            emulator.stack.push(-77);
                            emulator.ip += 1;
                            continue;
                        }

                        for i in 0..count {
                            if i >= (buffer.as_bytes().len() as i64) {
                                emulator.memory[buf as usize] = 0;
                            } else {
                                emulator.memory[buf as usize] = buffer.as_bytes()[i as usize];
                            }
                            buf += 1;
                        }

                        emulator.stack.push(count);
                    }
                    1 => {
                        // SYS_write
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

                        let mut bytes: Vec<u8> = vec![];
                        let mut i = 0;

                        while i < count {
                            bytes.push(emulator.memory[(buf + i) as usize]);
                            i += 1;
                        }

                        let mut buffer = String::new();
                        if let Ok(string) = String::from_utf8(bytes.clone()) {
                            buffer = string;
                        } else {
                            for b in bytes {
                                buffer.push(b as char);
                            }
                        };

                        if let Some(stream) = emulator.fds.get(&(fd as usize)) {
                            if let Err(_) = stream.write(buffer) {
                                emulator.stack.push(-77);
                            } else {
                                emulator.stack.push(count);
                            }
                        } else {
                            emulator.stack.push(-77);
                        }
                    }
                    3 => {
                        // SYS_close
                        let fd;

                        if let Some(f) = emulator.stack.pop() {
                            fd = f;
                        } else {
                            panic!("stack underflow");
                        }

                        if let Some(_) = emulator.fds.remove(&(fd as usize)) {
                            emulator.stack.push(-77);
                        }
                    }
                    60 => {
                        // SYS_exit
                        let code;
                        if let Some(c) = emulator.stack.pop() {
                            code = c as i32;
                        } else {
                            panic!("stack underflow");
                        }
                        exit(code);
                    }
                    257 => {
                        // SYS_openat
                        let dfd;
                        let nameptr;
                        let flags;
                        let mode;

                        if let Some(fd) = emulator.stack.pop() {
                            dfd = fd;
                        } else {
                            panic!("stack underflow");
                        }

                        if let Some(ptr) = emulator.stack.pop() {
                            nameptr = ptr;
                        } else {
                            panic!("stack underflow");
                        }

                        if let Some(fl) = emulator.stack.pop() {
                            flags = fl as u64;
                        } else {
                            panic!("stack underflow");
                        }

                        if let Some(r#mod) = emulator.stack.pop() {
                            mode = r#mod;
                        } else {
                            panic!("stack underflow");
                        }

                        if mode != 420 {
                            emulator.stack.push(-1);
                            emulator.ip += 1;
                            continue;
                        }

                        if dfd != -100 {
                            emulator.stack.push(-1);
                            emulator.ip += 1;
                            continue;
                        }

                        let mut filename = String::new();
                        {
                            let mut bytes: Vec<u8> = vec![];
                            for i in nameptr..(emulator.memory.len() as i64) {
                                if emulator.memory[i as usize] == 0 {
                                    break;
                                }

                                bytes.push(emulator.memory[i as usize]);
                            }

                            if let Ok(string) = String::from_utf8(bytes.clone()) {
                                filename = string;
                            } else {
                                for b in bytes {
                                    filename.push(b as char);
                                }
                            }
                        }

                        static O_CREAT: u64 = 64;
                        static O_RDONLY: u64 = 0;
                        static O_WRONLY: u64 = 1;
                        static O_TRUNC: u64 = 512;
                        static O_RDWR: u64 = 2;

                        #[rustfmt::skip] let istrunc =     if (flags & O_TRUNC) == O_TRUNC { true } else { false };
                        #[rustfmt::skip] let iswriteonly = if (flags & O_WRONLY) == O_WRONLY { true } else { false };
                        #[rustfmt::skip] let iscreate =    if (flags & O_CREAT) == O_CREAT { true } else { false };
                        #[rustfmt::skip] let isreadonly =  if (flags & O_RDONLY) == O_RDONLY { true } else { false };
                        #[rustfmt::skip] let isrdwr =      if (flags & O_RDWR) == O_RDWR { true } else { false };

                        if !istrunc && !iswriteonly && !iscreate && !isreadonly && isrdwr {
                            emulator.stack.push(-1);
                            emulator.ip += 1;
                            continue;
                        }

                        let file = if iswriteonly {
                            let f = OpenOptions::new()
                                .create(iscreate)
                                .truncate(istrunc)
                                .write(true)
                                .read(false)
                                .open(filename);
                            if f.is_err() {
                                emulator.stack.push(-2);
                                emulator.ip += 1;
                                continue;
                            }
                            f.unwrap()
                        } else if isreadonly {
                            let f = OpenOptions::new()
                                .create(iscreate)
                                .truncate(istrunc)
                                .write(false)
                                .read(true)
                                .open(filename);
                            if f.is_err() {
                                emulator.stack.push(-2);
                                emulator.ip += 1;
                                continue;
                            }
                            f.unwrap()
                        } else if isrdwr {
                            let f = OpenOptions::new()
                                .create(iscreate)
                                .truncate(istrunc)
                                .write(true)
                                .read(true)
                                .open(filename);
                            if f.is_err() {
                                emulator.stack.push(-2);
                                emulator.ip += 1;
                                continue;
                            }
                            f.unwrap()
                        } else {
                            panic!("unreachable");
                        };

                        let fd = emulator.fds.len();

                        let mut stream;
                        {
                            stream = Stream::new();
                            stream.file = Some(file);
                        }

                        emulator.fds.insert(fd, stream);
                        emulator.stack.push(fd as i64);
                    }
                    _ => panic!("unsupported syscall: {}", syscall_number),
                }
                emulator.ip += 1;
            }
            IrInstructionKind::AllocVariable => emulator.ip += 1,
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
            IrInstructionKind::Load16 => {
                let addr;
                if let Some(a) = emulator.stack.pop() {
                    addr = a;
                } else {
                    panic!("stack underflow");
                }

                let mut bytes: [u8; 2] = [0, 0];
                for i in 0..2 {
                    bytes[i] = emulator.memory[(addr as usize) + i];
                }
                emulator.stack.push(i16::from_le_bytes(bytes) as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::Store16 => {
                let mut addr;
                let value: i16;

                if let Some(a) = emulator.stack.pop() {
                    addr = a;
                } else {
                    panic!("stack underflow");
                }

                if let Some(v) = emulator.stack.pop() {
                    value = v as i16;
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
            IrInstructionKind::Load32 => {
                let addr;
                if let Some(a) = emulator.stack.pop() {
                    addr = a;
                } else {
                    panic!("stack underflow");
                }

                let mut bytes: [u8; 4] = [0, 0, 0, 0];
                for i in 0..4 {
                    bytes[i] = emulator.memory[(addr as usize) + i];
                }
                emulator.stack.push(i32::from_le_bytes(bytes) as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::Store32 => {
                let mut addr;
                let value: i32;

                if let Some(a) = emulator.stack.pop() {
                    addr = a;
                } else {
                    panic!("stack underflow");
                }

                if let Some(v) = emulator.stack.pop() {
                    value = v as i32;
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
                let value: i64;

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
                } else {
                    panic!("variable not found");
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
            IrInstructionKind::NotEqual => {
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

                emulator.stack.push((b != a) as i64);
                emulator.ip += 1;
            }
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
            IrInstructionKind::Greater => {
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

                emulator.stack.push((a > b) as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::LessEqual => {
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

                emulator.stack.push((a <= b) as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::GreaterEqual => {
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

                emulator.stack.push((a >= b) as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::AllocMemory => emulator.ip += 1,
            IrInstructionKind::PushMemory => {
                if let Some(addr) = emulator.memories.get(&(op.operand.integer as usize)) {
                    emulator.stack.push(*addr as i64);
                } else {
                    panic!("memory region not found");
                }
                emulator.ip += 1;
            }
            IrInstructionKind::ShiftLeft => {
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

                emulator.stack.push(a << b);
                emulator.ip += 1;
            }
            IrInstructionKind::ShiftRight => {
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

                emulator.stack.push(a >> b);
                emulator.ip += 1;
            }
            IrInstructionKind::Or => {
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

                emulator.stack.push(a | b);
                emulator.ip += 1;
            }
            IrInstructionKind::And => {
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

                emulator.stack.push(a & b);
                emulator.ip += 1;
            }
            IrInstructionKind::Not => {
                if let Some(v) = emulator.stack.pop() {
                    emulator.stack.push(!v);
                } else {
                    panic!("stack underflow");
                }
                emulator.ip += 1;
            }
            IrInstructionKind::PushString => {
                let string = escape_string(op.operand.string);
                if let Some(addr) = emulator.find_string_in_memory(string) {
                    emulator.stack.push(addr as i64);
                } else {
                    eprintln!("Unreachable");
                    exit(1);
                }
                emulator.ip += 1;
            }
            IrInstructionKind::Call => {
                emulator.ret_stack.push(emulator.ip + 1);
                emulator.ip = op.operand.integer as usize;
            }
            IrInstructionKind::Return => {
                if let Some(p) = emulator.ret_stack.pop() {
                    emulator.ip = p;
                } else {
                    panic!("return stack underflow");
                }
            }
            IrInstructionKind::CastPointer => emulator.ip += 1,
            IrInstructionKind::CastInt => emulator.ip += 1,
            IrInstructionKind::Argc => {
                emulator.stack.push(emulator.args.len() as i64);
                emulator.ip += 1;
            }
            IrInstructionKind::Argv => {
                emulator.stack.push(argv);
                emulator.ip += 1;
            }
            IrInstructionKind::Envp => {
                emulator.stack.push(envp);
                emulator.ip += 1;
            }
        }
    }
}

pub fn emulate_file(config: Config) -> Result<()> {
    let ir = compile_file_into_ir(config.clone().input)?;
    let mut emulator = Emulator::new();

    emulator.args.push(config.input);
    for a in config.run.args {
        emulator.args.push(a);
    }

    emulator.init(ir.clone());
    emulate_program(ir.clone(), &mut emulator);
    Ok(())
}
