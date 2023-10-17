use std::{
    io::{self, Write},
    path::PathBuf,
};

pub trait S {
    fn pin(&self) -> String;
    fn bold(&self) -> String;
    fn yellow(&self) -> String;
}

impl S for str {
    fn pin(&self) -> String {
        format!("\r{}\x1B[K", self)
    }

    fn bold(&self) -> String {
        format!("\x1b[1m{}\x1b[0m", self)
    }

    fn yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[39m", self)
    }
}

pub trait P {
    fn p_display(&self) -> String;
}

impl P for PathBuf {
    fn p_display(&self) -> String {
        if self.is_absolute() || self.starts_with("./") {
            self.display().to_string()
        } else {
            format!("./{}", self.display())
        }
    }
}

pub fn confirm(prompt: &str) -> bool {
    print!("{} › (y/N) ", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    input.trim().to_lowercase() == "y"
}
