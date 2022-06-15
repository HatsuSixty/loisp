mod instructions;
mod types;
mod ir;
mod lexer;
mod parser;

use instructions::*;
use lexer::*;
use ir::*;
use parser::*;

use std::fs;

fn main() -> Result<(), LoispError> {
    let file = "test.loisp".to_string();
    let program = fs::read_to_string(file.as_str())?;

    let lexer = Lexer::from_chars(program.chars(), file);
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

    ir.to_nasm_linux_x86_64_assembly("output.asm".to_string())?;
    Ok(())
}
