use super::emulator::*;
use super::instructions::*;
use super::ir::*;

use std::io;
use std::io::Write;
use std::process::exit;

pub fn evaluate_line(
    line: String,
    context: &mut LoispContext,
    emulator: &mut Emulator,
) -> Result<(), LoispError> {
    if line == "" || line == "\n" {
        return Ok(());
    }

    let ir = compile_string_into_ir(line, context, "<stdin>".to_string())?;
    emulator.init(ir.clone());
    emulator.ip = 0;
    emulate_program(ir.clone(), emulator);

    println!("===> Last value on the stack");

    let value;
    if let Some(v) = emulator.stack.last() {
        value = format!("{}", v);
    } else {
        value = "<none>".to_string();
    }

    println!("--> {}", value);

    Ok(())
}

pub fn start_repl() {
    let mut context = LoispContext::new();
    let mut emulator = Emulator::new();

    loop {
        let mut line = String::new();
        print!(">>> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line).expect("error");

        if line.trim() == "quit".to_string() || line.trim() == "q".to_string() {
            exit(0);
        } else if let Err(error) = evaluate_line(line, &mut context, &mut emulator) {
            eprintln!("{}", error);
        }
    }
}
