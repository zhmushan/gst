use std::path::PathBuf;

pub trait S {
    fn pin(&self) -> String;
    fn bold(&self) -> String;
}

impl S for str {
    fn pin(&self) -> String {
        format!("\r{}\x1B[K", self)
    }

    fn bold(&self) -> String {
        format!("\x1b[1m{}\x1b[0m", self)
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
