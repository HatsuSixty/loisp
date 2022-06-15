mod instructions;
mod types;
mod ir;
mod lexer;
mod parser;

use instructions::*;
use lexer::*;
use ir::*;
use parser::*;

fn main() -> Result<(), LoispError> {
    let program = "(print (+ 34 35))\n(+ 34 35)\nduawudi";

    let lexer = Lexer::from_chars(program.chars(), "joao.txt".to_string());
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
