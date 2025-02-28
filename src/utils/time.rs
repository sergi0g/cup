// When you're too bored to type some things, you get this...

use std::time::SystemTime;

pub fn elapsed(start: SystemTime) -> u32 {
    start.elapsed().unwrap().as_millis() as u32
}

pub fn now() -> SystemTime {
    SystemTime::now()
}
