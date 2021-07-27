

use near_sdk::{
 Timestamp, Gas, Balance
};
pub type TimestampSec = u32;

pub const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;
pub const GAS_FOR_AFTER_FT_TRANSFER: Gas = 10_000_000_000_000;

pub const ONE_YOCTO: Balance = 1;
pub const NO_DEPOSIT: Balance = 0;

pub(crate) fn to_nano(timestamp: TimestampSec) -> Timestamp {
    Timestamp::from(timestamp) * 10u64.pow(9)
}