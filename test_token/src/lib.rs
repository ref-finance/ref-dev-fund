/*!
* Ref NEP-141 Token contract
*
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{ValidAccountId, U128};
// Needed by `impl_fungible_token_core` for old Rust.
#[allow(unused_imports)]
use near_sdk::env;
use near_sdk::{near_bindgen, log, AccountId, Balance, PanicOnDefault, PromiseOrValue};


near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub ft: FungibleToken,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Contract {
            ft: FungibleToken::new(b"a".to_vec()),
        }
    }

    pub fn mint(&mut self, amount: U128) {
        let account_id = env::signer_account_id();
        let amount: Balance = amount.into();
        self.ft.internal_deposit(&account_id, amount);
        log!("Mint {} token to {}", amount, account_id);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, ft);
near_contract_standards::impl_fungible_token_storage!(Contract, ft);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
         FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: String::from("Test Token"),
            symbol: String::from("TT"),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 18,
        }
    }
}
