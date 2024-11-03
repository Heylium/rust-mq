pub mod placement;
pub mod poll;

pub fn retry_times() -> usize {
    3
}

pub fn retry_sleep_time(times: usize) -> u64 {
    (times * 2usize) as u64
}