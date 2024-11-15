// Miscellaneous utility functions that are too small to go in a separate file

use chrono::Local;

/// Gets the current timestamp. Mainly exists so I don't have to type this one line of code ;-)
pub fn timestamp() -> i64 {
    Local::now().timestamp_millis()
}
