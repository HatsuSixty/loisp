#[derive(Clone)]
pub struct ConfigRun {
    pub run: bool,
    pub args: Vec<String>,
}

impl ConfigRun {
    pub fn new() -> ConfigRun {
        ConfigRun {
            run: false,
            args: vec![],
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub silent: bool,
    pub run: ConfigRun,
    pub piped: bool,
    pub emulate: bool,
    pub input: String,
    pub output: Option<String>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            silent: false,
            run: ConfigRun::new(),
            piped: false,
            input: String::new(),
            output: None,
            emulate: false,
        }
    }
}
