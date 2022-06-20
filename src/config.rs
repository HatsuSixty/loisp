#[derive(Clone)]
pub struct Config {
    pub silent: bool,
    pub run: bool,
    pub piped: bool,
    pub input: String,
    pub output: Option<String>
}

impl Config {
    pub fn new() -> Config {
        Config {
            silent: false,
            run: false,
            piped: false,
            input: String::new(),
            output: None
        }
    }
}
