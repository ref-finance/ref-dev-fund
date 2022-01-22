/*!
* REF session_vault contract
*
*/
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap};
use near_sdk::json_types::{ValidAccountId, WrappedBalance};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault};

pub use crate::views::ContractInfo;
use crate::account::VAccount;
mod owner;
mod account;
mod utils;
mod views;

near_sdk::setup_alloc!();

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Accounts,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractData {
    // owner of this contract
    owner_id: AccountId,

    // token kept by this vault
    token_account_id: AccountId,

    // the total realized amount in this vault
    total_balance: Balance,
    
    // already claimed balance
    claimed_balance: Balance,

    accounts: LookupMap<AccountId, VAccount>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VContractData {
    Current(ContractData),
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    data: VContractData,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: ValidAccountId, token_id: ValidAccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            data: VContractData::Current(ContractData {
                owner_id: owner_id.into(),
                token_account_id: token_id.into(),
                total_balance: 0,
                claimed_balance: 0,
                accounts: LookupMap::new(StorageKeys::Accounts)
            }),
        }
    }
}

impl Contract {
    fn data(&self) -> &ContractData {
        match &self.data {
            VContractData::Current(data) => data,
        }
    }

    fn data_mut(&mut self) -> &mut ContractData {
        match &mut self.data {
            VContractData::Current(data) => data,
        }
    }
}