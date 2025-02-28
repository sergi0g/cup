#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        eprintln!("\x1b[31;1mERROR\x1b[0m {}", format!($($arg)*));
        std::process::exit(1);
    })
}

/// This struct mostly exists so we can print stuff without passing debug or raw every time.
#[derive(Clone)]
pub struct Logger {
    debug: bool,
    raw: bool,
}

impl Logger {
    pub fn new(debug: bool, raw: bool) -> Self {
        Self { debug, raw }
    }

    pub fn warn(&self, msg: impl AsRef<str>) {
        if !self.raw {
            eprintln!("\x1b[33;1m WARN\x1b[0m {}", msg.as_ref());
        }
    }

    pub fn info(&self, msg: impl AsRef<str>) {
        if !self.raw {
            println!("\x1b[36;1m INFO\x1b[0m {}", msg.as_ref());
        }
    }

    pub fn debug(&self, msg: impl AsRef<str>) {
        if self.debug {
            println!("\x1b[35;1mDEBUG\x1b[0m {}", msg.as_ref());
        }
    }

    pub fn set_raw(&mut self, raw: bool) {
        self.raw = raw
    }
}
