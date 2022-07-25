use std::fs;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::str;

use super::common::*;
use super::print_info;

static LOISP_FILE_EXTENSION: &str = ".loisp";

#[derive(Debug)]
pub struct TestStats {
    failed: usize,
    ignored: usize,
    passed: usize
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {
    pub args: Vec<String>,
    pub stdout: String,
    pub stderr: String,
}

                                              // test      compiled
pub fn cmd_run_return_test_case(cmd: String) -> (TestCase, bool) {
    print_info!("CMD", "{}", cmd);

    let mut compiled = true;
    let mut test_case = TestCase {
        args: vec![],
        stdout: String::new(),
        stderr: String::new(),
    };

    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd.as_str())
        .output()
        .expect("Failed to run shell command");
    let exit_code = output.status.code();

    match exit_code {
        Some(code) => {
            print_info!("INFO", "Program exited with code `{}`", code);
            if code != 0 {
                compiled = false;
            }
        }
        None => {
            print_info!("INFO", "Program exited with signal");
            compiled = false;
        }
    }

    test_case.stdout = String::from_utf8(output.stdout).unwrap().trim().to_string();
    test_case.stderr = String::from_utf8(output.stderr).unwrap().trim().to_string();

    for (i, s) in cmd.trim().split(' ').enumerate() {
        if i >= 3 {
            test_case.args.push(s.to_string());
        }
    }

    (test_case.clone(), compiled)
}

pub fn read_file_return_test_case(file: String) -> io::Result<TestCase> {
    let mut test_case = TestCase {
        args: vec![],
        stdout: String::new(),
        stderr: String::new(),
    };

    let source = fs::read_to_string(file.as_str())?;

    let mut lines: Vec<String> = vec![];
    for s in source.trim().split('|') {
        lines.push(s.to_string());
    }

    for l in lines {
        let mut tokens: Vec<String> = vec![];
        for t in l.trim().split('=') {
            tokens.push(t.to_string());
        }
        assert!(
            tokens.len() == 2,
            "Parsing Error: missing `name` or `=`. Or more than 1 `=` encountered"
        );
        let name = &tokens[0];
        let value = &tokens[1];

        match name.as_str().trim() {
            "stdout" => test_case.stdout = value.trim().to_string(),
            "stderr" => test_case.stderr = value.trim().to_string(),
            "args" => {
                let mut args: Vec<String> = vec![];
                for a in value.trim().split(' ') {
                    args.push(a.to_string());
                }
                test_case.args = args;
            }
            _ => assert!(false, "Parsing Error: unknown variable encountered"),
        }
    }

    Ok(test_case.clone())
}

pub fn save_test_case_in_conf_file(test: TestCase, file: String) -> io::Result<()> {
    print_info!("INFO", "Saving output to `{}`", file);

    let f = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(file)?;

    let mut buffer = BufWriter::new(f);

    write!(
        buffer,
        "stdout = {}|stderr = {}",
        test.stdout.trim(),
        test.stderr.trim()
    )?;
    write!(buffer, "|args =")?;
    for a in test.args {
        write!(buffer, " {}", a)?;
    }

    buffer.flush()?;
    Ok(())
}

pub fn save_tests_for_folder(folder: String) -> io::Result<()> {
    print_info!("INFO", "Saving tests for folder `{}`", folder);

    let dir = fs::read_dir(folder)?;
    let mut paths: Vec<String> = vec![];

    for path in dir {
        paths.push(format!("{}", path?.path().display()));
    }

    for p in paths {
        if p.ends_with(LOISP_FILE_EXTENSION) {
            let (tc, _) = cmd_run_return_test_case(format!("./target/debug/loisp -s run {}", p));
            let tc_output = format!("{}.conf", file_name_without_extension(p));
            save_test_case_in_conf_file(tc, tc_output)?;
            println!();
        }
    }
    Ok(())
}

pub fn run_tests_for_folder(folder: String) -> io::Result<()> {
    print_info!("INFO", "Running tests for folder `{}`", folder);

    let mut stats = TestStats {
        passed: 0,
        failed: 0,
        ignored: 0,
    };
    let dir = fs::read_dir(folder)?;
    let mut paths: Vec<String> = vec![];

    for path in dir {
        paths.push(format!("{}", path?.path().display()));
    }

    for p in paths {
        if p.ends_with(LOISP_FILE_EXTENSION) {
            let expected_path = format!("{}.conf", file_name_without_extension(p.clone()));
            let (got, compiled) =
                cmd_run_return_test_case(format!("./target/debug/loisp -s run {}", p.clone()));
            if !Path::new(expected_path.as_str()).exists() {
                print_info!(
                    "WARN",
                    "No output found for `{}`, only testing if it compiles",
                    p.clone()
                );
                if !compiled {
                    print_info!("ERROR", "Test not compiled");
                    stats.failed += 1;
                } else {
                    stats.passed += 1;
                }
                stats.ignored += 1;
            } else {
                let expected = read_file_return_test_case(expected_path)?;

                if expected != got {
                    print_info!("ERROR", "Test failed:\n    Expected: {:#?}\n    Got: {:#?}", expected, got);
                    stats.failed += 1;
                } else {
                    stats.passed += 1;
                }
            }
            println!();
        }
    }

    print_info!("STAT", "{:#?}", stats);

    Ok(())
}
