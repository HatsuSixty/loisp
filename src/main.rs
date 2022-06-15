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
    let program = "(print (+ 34 35))\n(+ 34 35)";

    let lexer = Lexer::from_chars(program.chars(), "joao.txt".to_string());
    let instructions = construct_instructions_from_tokens(&mut lexer.peekable())?;
    let mut ir = IrProgram::new();

    for i in instructions {
        i.to_ir(&mut ir)?;
    }

    ir.to_nasm_linux_x86_64_assembly("output.asm".to_string())?;
    Ok(())
}
