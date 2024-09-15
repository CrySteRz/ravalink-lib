use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn get_unix_timestamp() -> Duration {
    let start = SystemTime::now();
    start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}

pub fn get_timestamp() -> u64 {
    get_unix_timestamp().as_secs()
}
