#[derive(Clone)]
pub struct Config {
    pub silent: bool,
    pub run: bool,
    pub input: String,
    pub output: String
}

impl Config {
    pub fn new() -> Config {
        Config {
            silent: false,
            run: false,
            input: String::new(),
            output: String::new()
        }
    }
}
