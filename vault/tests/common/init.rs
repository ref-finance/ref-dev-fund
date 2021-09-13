// use std::collections::HashMap;
use std::convert::TryFrom;

// use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::AccountId;
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount,
};

use vault::{ContractContract as Vault, Stats, AccountOutput};

use test_token::ContractContract as TestToken;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TEST_TOKEN_WASM_BYTES => "../res/test_token.wasm",
    VAULT_WASM_BYTES => "../res/vault_release.wasm",
}

pub fn test_token(
    root: &UserAccount,
    token_id: AccountId,
    accounts_to_register: Vec<AccountId>,
) -> ContractAccount<TestToken> {
    let t = deploy!(
        contract: TestToken,
        contract_id: token_id,
        bytes: &TEST_TOKEN_WASM_BYTES,
        signer_account: root
    );
    call!(root, t.new()).assert_success();

    for account_id in accounts_to_register {
        call!(
            root,
            t.storage_deposit(Some(to_va(account_id)), None),
            deposit = to_yocto("1")
        )
        .assert_success();
    }
    t
}

pub fn balance_of(token: &ContractAccount<TestToken>, account_id: &AccountId) -> u128 {
    view!(token.ft_balance_of(to_va(account_id.clone())))
        .unwrap_json::<U128>()
        .0
}

pub fn assert_stats(stats: &Stats, current_round: u32, claimed_balance: u128, locked_balance: u128, liquid_balance: u128, unclaimed_balance: u128) {
    assert_eq!(stats.current_round, current_round);
    assert_eq!(stats.claimed_balance.0, claimed_balance);
    assert_eq!(stats.locked_balance.0, locked_balance);
    assert_eq!(stats.liquid_balance.0, liquid_balance);
    assert_eq!(stats.unclaimed_balance.0, unclaimed_balance);
}

pub fn assert_userinfo(info: &AccountOutput, last_claim_round: u32, unclaimed_amount: u128) {
    assert_eq!(info.last_claim_round, last_claim_round);
    assert_eq!(info.unclaimed_amount.0, unclaimed_amount);
}


pub fn to_va(a: AccountId) -> ValidAccountId {
    ValidAccountId::try_from(a).unwrap()
}

pub fn setup_vault(total: u128, start_at: u32, interval: u32, rounds: u32) -> (
    UserAccount,
    UserAccount,
    ContractAccount<Vault>,
    ContractAccount<TestToken>,
) {
    let root = init_simulator(None);
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let vault = deploy!(
        contract: Vault,
        contract_id: "vault".to_string(),
        bytes: &VAULT_WASM_BYTES,
        signer_account: root,
        init_method: new(
            to_va("owner".to_string()), 
            to_va("test_token".to_string()),
            U128(total),  // total balance
            start_at,  // start timestamp
            interval,  // release interval
            rounds  // release round
        )
    );
    let token = test_token(&root, "test_token".to_string(), vec!["vault".to_string(), owner.account_id()]);

    call!(owner, token.mint(U128(total))).assert_success();

    call!(
        owner,
        token.ft_transfer(to_va("vault".to_string()), U128(total), None),
        deposit = 1
    ).assert_success();

    (root, owner, vault, token)
}
