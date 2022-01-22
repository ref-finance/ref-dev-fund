use near_sdk_sim::{ExecutionResult};

pub fn get_error_count(r: &ExecutionResult) -> u32 {
    r.promise_errors().len() as u32
}

pub fn get_error_status(r: &ExecutionResult) -> String {
    format!("{:?}", r.promise_errors()[0].as_ref().unwrap().status())
}

pub(crate) fn to_nano(timestamp: u32) -> u64 {
    u64::from(timestamp) * 10u64.pow(9)
}