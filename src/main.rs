mod instructions;
mod types;
mod ir;
mod lexer;
mod parser;

use instructions::*;
use ir::*;

fn main() -> Result<(), LoispError> {
    let ir = compile_file_into_ir("test.loisp".to_string())?;

    ir.to_nasm_linux_x86_64_assembly("output.asm".to_string())?;
    Ok(())
}
