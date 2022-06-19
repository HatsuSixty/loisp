use super::config::*;

use std::process::{Command, Stdio};
use std::io;
use std::env;

#[macro_export]
macro_rules! print_info {
    () => {
        std::print!("\n")
    };
    ($($arg:tt)*) => {{
        print!("{}", format!("[INFO] {}\n", format!($($arg)*)));
    }};
}

pub fn run_command_with_info(cmd: String, config: Config) -> io::Result<()> {
    if !config.silent {
        print_info!("CMD: {}", cmd)
    }

    let mut shell_cmd = Command::new("sh");
    shell_cmd.args(["-c", cmd.as_str()]);

    let cwd = format!("{}", env::current_dir()?.display());
    shell_cmd.current_dir(cwd.as_str());

    shell_cmd.stderr(Stdio::inherit());
    shell_cmd.stdout(Stdio::inherit());

    shell_cmd.status().expect(format!("Command {} failed to execute", cmd).as_str());
    Ok(())
}
