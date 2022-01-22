use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::WrappedBalance;
use near_sdk::AccountId;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct ContractInfo {
    // only onwer can manage accounts
    pub owner_id: AccountId,
    // token kept by this vault
    pub token_account_id: AccountId,
    // the total realized amount in this vault
    pub total_balance: WrappedBalance,
    // already claimed balance
    pub claimed_balance: WrappedBalance,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct AccountInfo {
    pub account_id: AccountId,
    // session start time
    pub start_timestamp: u32,
    // per session lasts, eg: 90 days 
    pub session_interval: u32,
    // totally how many session, eg: 1
    pub session_num: u32,
    // the session index of previous claim, start from 1
    pub last_claim_session: u32,
    // expected total_amount = session_num * release_per_session
    pub release_per_session: WrappedBalance,
    // actually deposited amount for the user
    // each time ft_transfer_call would increase this one
    // and realized_total_amount should >= expected total_amount to make it valid
    pub realized_total_amount: WrappedBalance,
    // unclaimed amount
    pub unclaimed_amount: WrappedBalance,
}