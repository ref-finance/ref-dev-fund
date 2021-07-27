use crate::*;
use crate::utils::TimestampSec;
use near_sdk::json_types::{ValidAccountId, WrappedBalance};
use near_sdk::{
    env, ext_contract, is_promise_success, log, near_bindgen, AccountId, Balance, PromiseOrValue,
};
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;

#[near_bindgen]
impl Contract {
    pub fn set_owner(&mut self, owner_id: ValidAccountId) {
        self.assert_owner();
        self.owner_id = owner_id.into();
    }

    pub fn remove_account(&mut self, account_id: ValidAccountId) -> bool {
        self.assert_owner();
        self.internal_remove_account(account_id.into())
    }

    pub fn add_account(
        &mut self, 
        account_id: ValidAccountId,
        start_timestamp: TimestampSec,
        release_interval: TimestampSec,
        release_rounds: u32,
        release_per_round: WrappedBalance,
    ) -> bool {
        self.assert_owner();
        self.internal_add_account(
            account_id.into(), 
            start_timestamp, 
            release_interval, 
            release_rounds,
            release_per_round.into()
        )
    }

    pub fn payment(&mut self, receiver_id: ValidAccountId, amount: WrappedBalance) -> PromiseOrValue<bool> {
        self.assert_owner();
        let amount: Balance = amount.into();
        let account_id: AccountId = receiver_id.into();

        let (liquid_balance, unclaimed_balance) = self.cur_funding_balance();
        assert!(
            (amount + unclaimed_balance) <= liquid_balance,
            "The payment amount beyonds liquidity"
        );

        if amount > 0 {
            self.claimed_balance += amount;

            ext_fungible_token::ft_transfer(
                account_id.clone(),
                amount.into(),
                Some(format!(
                    "Payment {} balance from {}",
                    amount,
                    env::current_account_id()
                )),
                &self.token_account_id,
                ONE_YOCTO,
                GAS_FOR_FT_TRANSFER,
            )
            .then(ext_payment::after_payment_transfer(
                account_id,
                amount.into(),
                &env::current_account_id(),
                NO_DEPOSIT,
                GAS_FOR_AFTER_FT_TRANSFER,
            ))
            .into()
        } else {
            PromiseOrValue::Value(true)
        }
    }


    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "ERR_NOT_ALLOWED"
        );
    }
}

#[ext_contract(ext_payment)]
trait AccountPaymentCallbacks {
    fn after_payment_transfer(&mut self, account_id: AccountId, amount: WrappedBalance) -> bool;
}

trait AccountPaymentCallbacks {
    fn after_payment_transfer(&mut self, account_id: AccountId, amount: WrappedBalance) -> bool;
}

#[near_bindgen]
impl AccountPaymentCallbacks for Contract {
    #[private]
    fn after_payment_transfer(&mut self, account_id: AccountId, amount: WrappedBalance) -> bool {
        let promise_success = is_promise_success();
        if !promise_success {
            self.claimed_balance -= amount.0;
            log!(
                "Payment failed and rollback, account is {}, balance is {}",
                account_id,
                amount.0
            );
        } else {
            log!(
                "Payment succeed, account is {}, balance is {}",
                account_id,
                amount.0
            );
        }
        promise_success
    }
}


