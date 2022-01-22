use crate::*;
use crate::utils::*;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct ContractInfo {
    pub version: String,
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
    pub start_timestamp: TimestampSec,
    // per session lasts, eg: 90 days 
    pub session_interval: TimestampSec,
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

#[near_bindgen]
impl Contract {
    /// Return contract basic info
    pub fn contract_metadata(&self) -> ContractInfo{
        let current_state = self.data();
        ContractInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: current_state.owner_id.clone(),
            token_account_id: current_state.token_account_id.clone(),
            total_balance: current_state.total_balance.into(),
            claimed_balance: current_state.claimed_balance.into(),
        }
    }

    pub fn get_account(&self, account_id: ValidAccountId) -> Option<AccountInfo> {
        if let Some(vacc) = self.data().accounts.get(account_id.as_ref()) {
            match vacc {
                VAccount::Current(acc) => {
                    Some(AccountInfo {
                        account_id: acc.account_id.clone(),
                        start_timestamp: acc.start_timestamp,
                        session_interval: acc.session_interval,
                        session_num: acc.session_num,
                        last_claim_session: acc.last_claim_session,
                        release_per_session: acc.release_per_session.into(),
                        realized_total_amount: acc.realized_total_amount.into(),
                        unclaimed_amount: acc.unclaimed_amount(env::block_timestamp()).into(),
                    })
                }
            }
        } else {
            None
        }
    }
}
