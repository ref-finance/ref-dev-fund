
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use std::collections::HashMap;
use near_sdk::json_types::{ValidAccountId, WrappedBalance};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, PanicOnDefault, 
};
use account::Account;
use utils::*;
// for sim-test
pub use view::{Stats, AccountOutput};

mod account;
mod utils;
mod owner;
mod view;

uint::construct_uint! {
    pub struct U256(4);
}

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub token_account_id: AccountId,
    pub total_balance: Balance,
    pub start_timestamp: TimestampSec,
    pub release_interval: TimestampSec,
    pub release_rounds: u32,

    pub accounts: HashMap<AccountId, Account>,
    pub claimed_balance: Balance,
    // liquid_balance = total - locked - claimed
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        token_account_id: ValidAccountId,

        total_balance: WrappedBalance,
        start_timestamp: TimestampSec,
        release_interval: TimestampSec,
        release_rounds: u32,
    ) -> Self {
        Self {
            accounts: HashMap::new(),
            owner_id: owner_id.into(),
            token_account_id: token_account_id.into(),
            total_balance: total_balance.into(),
            start_timestamp,
            release_interval,
            release_rounds,
            claimed_balance: 0,
        }
    }

}


impl Contract {
    fn cur_round_and_total_unlock(&self) -> (u32, u128) {
        let cur_round = if env::block_timestamp() > to_nano(self.start_timestamp) {
            ((env::block_timestamp() - to_nano(self.start_timestamp))
                / to_nano(self.release_interval)) as u32
        } else {
            0
        };

        let unlocked = if cur_round < self.release_rounds {
            (U256::from(self.total_balance) * U256::from(cur_round)
                / U256::from(self.release_rounds))
            .as_u128()
        } else {
            self.total_balance
        };

        (cur_round, unlocked)
    }

    fn cur_funding_balance(&self) -> (u128, u128) {
        let (_, global_unlocked) = self.cur_round_and_total_unlock();
        let liquid_balance = global_unlocked - self.claimed_balance;
        let mut unclaimed = 0_u128;
        for account in self.accounts.values() {
            unclaimed += account.unclaimed_amount(env::block_timestamp());
        }
        (liquid_balance, unclaimed)
    }
}
