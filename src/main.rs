mod common;
mod config;
mod instructions;
mod ir;
mod lexer;
mod parser;
mod tests;
mod types;
mod emulator;
mod repl;

use config::*;
use instructions::*;
use ir::*;
use tests::*;
use emulator::*;
use repl::*;

use std::env;
use std::ffi::OsString;

fn usage(stderr: bool) {
    let help = "Usage: loisp [FLAGS] <SUBCOMMAND>
    Subcommands:
        build   <file>     Compile <file> into an executable
        run     <file>     Compile <file> into an executable and run the generated executable
        emulate <file>     Emulate <file>
        save-test <folder> Save test cases for each file in <folder>
        run-test  <folder> Run tests for each file in <folder>
        help               Prints this help to stdout and exits with 0 exit code
    Flags:
        -s             Do not show any output (except errors)
        -o <file>      Change the name of the executable that gets generated\n";
    if stderr {
        eprint!("{}", help)
    } else {
        print!("{}", help)
    }
}

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
        start_repl();
    } else {
        let mut run = false;
        let mut run_flags: Vec<String> = vec![];
        let mut silent = false;
        let mut piped = false;
        let mut emulate = false;
        let mut input = String::new();
        let mut output = None;
        while args.len() > 0 {
            if let Some(arg) = shift(&mut args) {
                match arg.as_str() {
                    "build" => {
                        if let Some(i) = shift(&mut args) {
                            input = i
                        } else {
                            usage(true);
                            eprintln!("ERROR: No input file was provided");
                            std::process::exit(1)
                        }
                        break;
                    }
                    "run" => {
                        if let Some(i) = shift(&mut args) {
                            input = i;
                            run = true
                        } else {
                            usage(true);
                            eprintln!("ERROR: No input file was provided");
                            std::process::exit(1)
                        }
                        while let Some(flag) = shift(&mut args) {
                            run_flags.push(flag);
                        }
                        break;
                    }
                    "emulate" => {
                        if let Some(i) = shift(&mut args) {
                            input = i;
                            emulate = true;
                        } else {
                            usage(true);
                            eprintln!("ERROR: No input file was provided");
                            std::process::exit(1);
                        }
                        while let Some(flag) = shift(&mut args) {
                            run_flags.push(flag);
                        }
                        break;
                    }
                    "save-test" => {
                        if let Some(i) = shift(&mut args) {
                            save_tests_for_folder(i)?;
                            std::process::exit(0);
                        } else {
                            usage(true);
                            eprintln!("ERROR: No input folder was provided");
                            std::process::exit(1);
                        }
                    }
                    "run-test" => {
                        if let Some(i) = shift(&mut args) {
                            run_tests_for_folder(i)?;
                            std::process::exit(0);
                        } else {
                            usage(true);
                            eprintln!("ERROR: No input folder was provided");
                            std::process::exit(1);
                        }
                    }
                    "help" => {
                        usage(false);
                        std::process::exit(0);
                    }
                    "-s" => {
                        silent = true;
                        piped = true
                    }
                    "-o" => {
                        if let Some(o) = shift(&mut args) {
                            output = Some(o)
                        } else {
                            usage(true);
                            eprintln!("ERROR: No output file was provided");
                            std::process::exit(1)
                        }
                    }
                    _ => {
                        usage(true);
                        eprintln!("ERROR: Unknown subcommand: {}", arg);
                        std::process::exit(1)
                    }
                }
            }
        }

        let mut config = Config::new();
        config.run.run = run;
        config.run.args = run_flags;
        config.silent = silent;
        config.piped = piped;
        config.output = output;
        config.input = input;
        config.emulate = emulate;

        if !config.silent && !config.emulate {
            print_info!("INFO", "Compiling `{}`", config.input);
        }

        if !config.emulate {
            compile_file_into_executable(config)?;
        } else {
            emulate_file(config)?;
        }
    }

    Ok(())
}
