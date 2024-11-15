// Logging utilites

/// This macro is an alternative to panic. It prints the message you give it and exits the process with code 1, without printing a stack trace. Useful for when the program has to exit due to a user error or something unexpected which is unrelated to the program (e.g. a failed web request)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        eprintln!("\x1b[38:5:204mERROR \x1b[0m {}", format!($($arg)*));
        std::process::exit(1);
    })
}

// A small macro to print in yellow as a warning
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        eprintln!("\x1b[38:5:192mWARN \x1b[0m {}", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        println!("\x1b[38:5:86mINFO \x1b[0m {}", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        println!("\x1b[38:5:63mDEBUG \x1b[0m {}", format!($($arg)*));
    })
}
