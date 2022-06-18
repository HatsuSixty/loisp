mod instructions;
mod types;
mod ir;
mod lexer;
mod parser;
mod common;

use instructions::*;
use ir::*;

fn main() -> Result<(), LoispError> {
    compile_file_into_executable("test.loisp", "output")?;
    Ok(())
}
