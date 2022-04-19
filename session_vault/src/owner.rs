//! Implement all the relevant logic for owner of this contract.
use crate::*;
use crate::utils::TimestampSec;
use near_sdk::{assert_one_yocto, StorageUsage, Promise};

impl Contract {
    /// Check how much storage taken costs and refund the left over back.
    fn internal_check_storage(&self, prev_storage: StorageUsage) {
        let storage_cost = env::storage_usage()
            .checked_sub(prev_storage)
            .unwrap_or_default() as Balance
            * env::storage_byte_cost();

        let refund = env::attached_deposit()
            .checked_sub(storage_cost)
            .expect(
                format!(
                    "ERR_STORAGE_DEPOSIT need {}, attatched {}", 
                    storage_cost, env::attached_deposit()
                ).as_str()
            );
        if refund > 0 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Change owner. Only can be called by owner.
    #[payable]
    pub fn set_owner(&mut self, owner_id: ValidAccountId) {
        self.assert_owner();
        assert_one_yocto();
        self.data_mut().owner_id = owner_id.as_ref().clone();
    }

    // /// Get the owner of this account.
    // pub fn get_owner(&self) -> AccountId {
    //     self.data().owner_id.clone()
    // }

    #[payable]
    pub fn add_account(
        &mut self,
        account_id: ValidAccountId,
        start_timestamp: TimestampSec,
        session_interval: TimestampSec,
        session_num: u32,
        release_per_session: WrappedBalance,
    ) -> bool {
        let prev_storage = env::storage_usage();
        self.assert_owner();
        let ret = self.internal_add_account(
            account_id.into(),
            start_timestamp,
            session_interval,
            session_num,
            release_per_session.into(),
        );
        self.internal_check_storage(prev_storage);
        ret
    }

    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.data().owner_id,
            "ERR_NOT_ALLOWED"
        );
    }

    /// Migration function.
    /// For next version upgrades, change this function.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let prev: Contract = env::state_read().expect("ERR_NOT_INITIALIZED");
        prev
    }
}

#[cfg(target_arch = "wasm32")]
mod upgrade {
    use near_sdk::env::BLOCKCHAIN_INTERFACE;
    use near_sdk::Gas;

    use super::*;

    const BLOCKCHAIN_INTERFACE_NOT_SET_ERR: &str = "Blockchain interface not set.";

    /// Gas for calling migration call.
    pub const GAS_FOR_MIGRATE_CALL: Gas = 10_000_000_000_000;

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub extern "C" fn upgrade() {
        env::setup_panic_hook();
        env::set_blockchain_interface(Box::new(near_blockchain::NearBlockchain {}));
        let contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_id = env::current_account_id().into_bytes();
        let method_name = "migrate".as_bytes().to_vec();
        unsafe {
            BLOCKCHAIN_INTERFACE.with(|b| {
                // Load input into register 0.
                b.borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .input(0);
                let promise_id = b
                    .borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
                b.borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
                let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_MIGRATE_CALL;
                b.borrow()
                    .as_ref()
                    .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                    .promise_batch_action_function_call(
                        promise_id,
                        method_name.len() as _,
                        method_name.as_ptr() as _,
                        0 as _,
                        0 as _,
                        0 as _,
                        attached_gas,
                    );
            });
        }
    }
}
