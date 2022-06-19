mod instructions;
mod types;
mod ir;
mod lexer;
mod parser;
mod common;
mod config;

use instructions::*;
use ir::*;
use config::*;

use std::env;
use std::ffi::OsString;

fn shift(args: &mut Vec<OsString>) -> Option<String> {
    if args.len() < 1 {
        return None;
    }
    let r = args[0].clone();
    args.remove(0);
    Some(r.to_str().unwrap().to_string())
}

fn main() -> Result<(), LoispError> {
    let mut args: Vec<OsString> = env::args_os().collect();
    shift(&mut args);

    if args.len() < 1 {
        eprintln!("ERROR: No subcommand was provided");
        std::process::exit(1)
    }

    let mut run = false;
    let mut silent = false;
    let mut input = String::new();
    let mut output = "output".to_string();
    while args.len() > 0 {
        if let Some(arg) = shift(&mut args) {
            match arg.as_str() {
                "build" => {
                    if let Some(i) = shift(&mut args) {
                        input = i
                    } else {
                        eprintln!("ERROR: No input file was provided");
                        std::process::exit(1)
                    }
                    break
                }
                "run" => {
                    if let Some(i) = shift(&mut args) {
                        input = i;
                        run = true
                    } else {
                        eprintln!("ERROR: No input file was provided");
                        std::process::exit(1)
                    }
                    break
                }
                "-s" => {
                    silent = true
                }
                "-o" => {
                    if let Some(o) = shift(&mut args) {
                        output = o
                    } else {
                        eprintln!("ERROR: No output file was provided");
                        std::process::exit(1)
                    }
                }
                _ => {
                    eprintln!("ERROR: Unknown subcommand: {}", arg);
                    std::process::exit(1)
                }
            }
        }
    }

    let mut config = Config::new();
    config.run = run;
    config.output = output;
    config.input = input;
    config.silent = silent;

    print_info!("Compiling `{}`", config.input);

    compile_file_into_executable(config)?;
    Ok(())
}
