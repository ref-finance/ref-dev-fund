use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use near_sdk::json_types::{WrappedBalance};
use near_sdk::{
    env, ext_contract, is_promise_success, log, near_bindgen, AccountId, Balance, PromiseOrValue,
};
// use std::cmp::Ordering;

use crate::utils::*;
use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub account_id: AccountId,
    pub start_timestamp: TimestampSec,
    pub release_interval: TimestampSec,
    pub release_rounds: u32,
    pub last_claim_round: u32,
    pub release_per_round: Balance,
}

impl Account {
    pub fn unclaimed_amount(&self, cur_ts: u64) -> u128 {
        if self.last_claim_round >= self.release_rounds {
            return 0_u128;
        }

        let cur_round = if cur_ts > to_nano(self.start_timestamp) {
            ((cur_ts - to_nano(self.start_timestamp))
                / to_nano(self.release_interval)) as u32
        } else {
            0
        };

        let times = if cur_round >= self.release_rounds {
            self.release_rounds - self.last_claim_round
        } else {
            cur_round - self.last_claim_round
        };

        let amount = self.release_per_round * times as u128;

        amount
    }
}

#[near_bindgen]
impl Contract {

    pub fn claim(&mut self) -> PromiseOrValue<bool> {
        let account_id = env::predecessor_account_id();

        let (_, global_unlocked) = self.cur_round_and_total_unlock();
        let liquid_balance = global_unlocked - self.claimed_balance;

        let account = self.accounts.get(&account_id).expect("Account not exist in this contract");
        let amount = account.unclaimed_amount(env::block_timestamp());
        if amount == 0 {
            return PromiseOrValue::Value(true);
        }

        assert!(
            amount <= liquid_balance,
            "The claim amount beyonds liquidity"
        );

        let times = (amount / account.release_per_round) as u32;
        let mut account = self.accounts.get_mut(&account_id).unwrap();

        self.claimed_balance += amount;
        account.last_claim_round += times;

        ext_fungible_token::ft_transfer(
            account_id.clone(),
            amount.into(),
            Some(format!(
                "Claiming unlocked {} balance from {}",
                amount,
                env::current_account_id()
            )),
            &self.token_account_id,
            ONE_YOCTO,
            GAS_FOR_FT_TRANSFER,
        )
        .then(ext_self::after_ft_transfer(
            account_id,
            amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_AFTER_FT_TRANSFER,
        ))
        .into()
    }
}


impl Contract {

    pub fn internal_remove_account(&mut self, account_id: AccountId) -> bool {
        if let Some(_) = self.accounts.remove(&account_id) {
            true
        } else {
            false
        }
    }

    pub fn internal_add_account(
        &mut self, 
        account_id: AccountId,
        start_timestamp: TimestampSec,
        release_interval: TimestampSec,
        release_rounds: u32,
        release_per_round: Balance,
    ) -> bool {
        if self.accounts.contains_key(&account_id) {
            false
        } else {
            let account = Account {
                account_id: account_id.clone(),
                start_timestamp,
                release_interval,
                release_rounds,
                last_claim_round: 0_u32,
                release_per_round,
            };
            self.accounts.insert(account_id, account);
            true
        }
    }
}

#[ext_contract(ext_self)]
trait AccountClaimCallbacks {
    fn after_ft_transfer(&mut self, account_id: AccountId, amount: WrappedBalance) -> bool;
}

trait AccountClaimCallbacks {
    fn after_ft_transfer(&mut self, account_id: AccountId, amount: WrappedBalance) -> bool;
}

#[near_bindgen]
impl AccountClaimCallbacks for Contract {
    #[private]
    fn after_ft_transfer(&mut self, account_id: AccountId, amount: WrappedBalance) -> bool {
        let promise_success = is_promise_success();
        if !promise_success {
            let mut account = self
                .accounts
                .get_mut(&account_id)
                .expect("The claim is not found");
            let times = (amount.0 / account.release_per_round) as u32;
            account.last_claim_round -= times;
            self.claimed_balance -= amount.0;
            log!(
                "Account claim failed and rollback, account is {}, balance is {}",
                account_id,
                amount.0
            );
        } else {
            log!(
                "Account claim succeed, account is {}, balance is {}",
                account_id,
                amount.0
            );
        }
        promise_success
    }
}
