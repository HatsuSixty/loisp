use super::config::*;

use std::process::{Command, Stdio};
use std::io;
use std::env;

#[macro_export]
macro_rules! print_info {
    ($p:literal,$($arg:tt)*) => {{
        print!("{}", format!("[{}] {}\n", $p, format!($($arg)*)));
    }};
}

pub fn file_name_without_extension(f: String) -> String {
    let mut input_file_extension = String::new();
    for c in f.chars().rev() {
        input_file_extension.insert(0, c);
        if c == '.' {
            break;
        }
    }
    f.as_str().replace(input_file_extension.as_str().trim(), "")
}

pub fn run_command_with_info(cmd: String, config: Config) -> io::Result<()> {
    if !config.silent {
        print_info!("CMD", "{}", cmd)
    }

    let mut shell_cmd = Command::new("sh");
    shell_cmd.args(["-c", cmd.as_str()]);

    let cwd = format!("{}", env::current_dir()?.display());
    shell_cmd.current_dir(cwd.as_str());

    if !config.piped {
        shell_cmd.stdout(Stdio::inherit());
    } else {
        shell_cmd.stdout(Stdio::null());
    }
    shell_cmd.stderr(Stdio::inherit());

    shell_cmd.status().expect(format!("Command {} failed to execute", cmd).as_str());
    Ok(())
}
