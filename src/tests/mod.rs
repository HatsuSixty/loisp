use std::fs;
use std::io;
use std::process::Command;
use std::str;
use std::io::BufWriter;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct TestCase {
    pub args: Vec<String>,
    pub stdout: String,
    pub stderr: String,
}

#[allow(dead_code)]
pub fn cmd_run_return_test_case(cmd: String) -> TestCase {
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

    test_case.stdout = str::from_utf8(&output.stdout[..]).unwrap().to_string();
    test_case.stderr = str::from_utf8(&output.stderr[..]).unwrap().to_string();

    for s in cmd.trim().split(' ') {
        test_case.args.push(s.to_string());
    }

    test_case.clone()
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn save_test_case_in_conf_file(test: TestCase, file: String) -> io::Result<()> {
    let f = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(file)?;

    let mut buffer = BufWriter::new(f);

    write!(buffer, "stdout = {}|stderr = {}", test.stdout.trim(), test.stderr.trim())?;
    write!(buffer, "|args =")?;
    for a in test.args {
        write!(buffer, " {}", a)?;
    }

    buffer.flush()?;
    Ok(())
}
