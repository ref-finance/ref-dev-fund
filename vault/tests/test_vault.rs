// use std::collections::HashMap;
use std::convert::TryFrom;

// use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::AccountId;
use near_sdk_sim::transaction::ExecutionStatus;
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, view, ContractAccount, ExecutionResult, UserAccount,
};

use vault::{ContractContract as Vault, Stats, AccountOutput};

use test_token::ContractContract as TestToken;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TEST_TOKEN_WASM_BYTES => "../res/test_token.wasm",
    VAULT_WASM_BYTES => "../res/vault_release.wasm",
}

pub fn should_fail(r: ExecutionResult) {
    println!("{:?}", r.status());
    match r.status() {
        ExecutionStatus::Failure(_) => {}
        _ => panic!("Should fail"),
    }
}

pub fn show_promises(r: ExecutionResult) {
    for promise in r.promise_results() {
        println!("{:?}", promise);
    }
}

fn test_token(
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
    // call!(
    //     root,
    //     t.mint(to_va(root.account_id.clone()), to_yocto("1000").into())
    // )
    // .assert_success();
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

fn balance_of(token: &ContractAccount<TestToken>, account_id: &AccountId) -> u128 {
    view!(token.ft_balance_of(to_va(account_id.clone())))
        .unwrap_json::<U128>()
        .0
}

fn assert_stats(stats: &Stats, current_round: u32, claimed_balance: u128, locked_balance: u128, liquid_balance: u128, unclaimed_balance: u128) {
    assert_eq!(stats.current_round, current_round);
    assert_eq!(stats.claimed_balance.0, claimed_balance);
    assert_eq!(stats.locked_balance.0, locked_balance);
    assert_eq!(stats.liquid_balance.0, liquid_balance);
    assert_eq!(stats.unclaimed_balance.0, unclaimed_balance);
}

fn assert_userinfo(info: &AccountOutput, last_claim_round: u32, unclaimed_amount: u128) {
    assert_eq!(info.last_claim_round, last_claim_round);
    assert_eq!(info.unclaimed_amount.0, unclaimed_amount);
}


fn to_va(a: AccountId) -> ValidAccountId {
    ValidAccountId::try_from(a).unwrap()
}

fn setup_vault() -> (
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
            U128(10000),  // total balance
            50,  // start timestamp
            10,  // release interval
            10  // release round
        )
    );
    let token = test_token(&root, "test_token".to_string(), vec!["vault".to_string()]);


    call!(
        owner,
        token.storage_deposit(None, None),
        deposit = to_yocto("1")
    )
    .assert_success();

    call!(
        owner,
        token.mint(U128(20000))
    )
    .assert_success();

    call!(
        owner,
        token.ft_transfer(to_va("vault".to_string()), U128(10000), None),
        deposit = 1
    )
    .assert_success();

    (root, owner, vault, token)
}

#[test]
fn test_vault_phase1() {
    println!("*** deploy and init contracts");
    let (root, owner, vault, token) = setup_vault();
    println!("block env ----> height: {}, ts: {}", 
        root.borrow_runtime().current_block().block_height,
        root.borrow_runtime().current_block().block_timestamp,
    );
    // height 26, 
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    // println!("Vault stats: {:#?}", vault_stats);
    assert_eq!(
        vault_stats,
        Stats {
            version: "0.2.1".to_string(),
            owner_id: owner.account_id(),
            token_account_id: token.account_id(),
            total_balance: U128(10000),
            start_timestamp: 50,
            release_interval: 10,
            release_rounds: 10,

            claimed_balance: U128(0),
            locked_balance: U128(10000),
            liquid_balance: U128(0),
            unclaimed_balance: U128(0),
            current_round: 0,
        }
    );

    println!("*** Chain goes for 60 blocks ***");
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 3, 0, 7000, 3000, 0);

    println!("*** Add user1");
    let user1 = root.create_user("user1".to_string(), to_yocto("10"));
    call!(user1, token.storage_deposit(None, None), deposit = to_yocto("1"))
    .assert_success();
    call!(owner, vault.add_account(to_va(user1.account_id()), 50, 10, 10, U128(20)), deposit = 0)
    .assert_success();
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 4, 0, 6000, 4000, 80);

    println!("*** User1 claim");
    call!(user1, vault.claim(), deposit = 0)
    .assert_success();
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 5, 80, 5000, 4920, 20);
    assert_eq!(balance_of(&token, &user1.account_id()), 80);
    let user_info = view!(vault.get_account(to_va(user1.account_id()))).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 4, 20);

    println!("*** Add user2 without storage_deposit to token");
    let user2 = root.create_user("user2".to_string(), to_yocto("10"));
    call!(owner, vault.add_account(to_va(user2.account_id()), 50, 10, 10, U128(20)), deposit = 0)
    .assert_success();
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 5, 80, 5000, 4920, 120);

    println!("*** User2 try to claim but fail");
    let withdrawal_result = call!(user2, vault.claim(), deposit = 0);
    let promise_errors = withdrawal_result.promise_errors();
    assert_eq!(promise_errors.clone().len(), 1, "Expected 1 failed promise when withdrawing to a fungible token to an unregistered account.");
    let promise_failure_opt = promise_errors.get(0).unwrap();
    let promise_failure = promise_failure_opt.as_ref().unwrap();
    if let ExecutionStatus::Failure(err) = promise_failure.status() {
        // At this time, this is the way to check for error messages.
        // This error comes from the fungible token contract.
        assert_eq!(
            err.to_string(),
            "Action #0: Smart contract panicked: The account user2 is not registered"
        );
    } else {
        panic!("Expected failure when claim to unregistered account.");
    }
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 6, 80, 4000, 5920, 160);
    assert_eq!(balance_of(&token, &user2.account_id()), 0);
    let user_info = view!(vault.get_account(to_va(user1.account_id()))).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 4, 40);
    let user_info = view!(vault.get_account(to_va(user2.account_id()))).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 0, 120);

    println!("*** User2 claim after storage_deposit");
    call!(user2, token.storage_deposit(None, None), deposit = to_yocto("1"))
    .assert_success();
    call!(user2, vault.claim(), deposit = 0)
    .assert_success();
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 7, 200, 3000, 6800, 80);
    assert_eq!(balance_of(&token, &user2.account_id()), 120);
    let user_info = view!(vault.get_account(to_va(user1.account_id()))).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 4, 60);
    let user_info = view!(vault.get_account(to_va(user2.account_id()))).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 6, 20);

    println!("*** Payment 1000 to user3 but without storage_deposit to token");
    let user3 = root.create_user("user3".to_string(), to_yocto("10"));
    let withdrawal_result = call!(owner, vault.payment(to_va(user3.account_id()), U128(1000)), deposit = 0);
    let promise_errors = withdrawal_result.promise_errors();
    assert_eq!(promise_errors.clone().len(), 1, "Expected 1 failed promise when withdrawing to a fungible token to an unregistered account.");
    let promise_failure_opt = promise_errors.get(0).unwrap();
    let promise_failure = promise_failure_opt.as_ref().unwrap();
    if let ExecutionStatus::Failure(err) = promise_failure.status() {
        // At this time, this is the way to check for error messages.
        // This error comes from the fungible token contract.
        assert_eq!(
            err.to_string(),
            "Action #0: Smart contract panicked: The account user3 is not registered"
        );
    } else {
        panic!("Expected failure when payment to unregistered account.");
    }
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 7, 200, 3000, 6800, 80);
    assert_eq!(balance_of(&token, &user3.account_id()), 0);

    println!("*** Payment 1000 again to user3 after storage_deposit");
    call!(user3, token.storage_deposit(None, None), deposit = to_yocto("1"))
    .assert_success();
    call!(owner, vault.payment(to_va(user3.account_id()), U128(1000)), deposit = 0)
    .assert_success();
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 8, 1200, 2000, 6800, 120);
    assert_eq!(balance_of(&token, &user3.account_id()), 1000);

    println!("*** Payment 6800 which exceeds liquidity");
    let payment_result = call!(owner, vault.payment(to_va(user3.account_id()), U128(6800)), deposit = 0);
    if let ExecutionStatus::Failure(err) = payment_result.status() {
        // At this time, this is the way to check for error messages.
        // This error comes from the fungible token contract.
        assert_eq!(
            err.to_string(),
            "Action #0: Smart contract panicked: panicked at \'The payment amount beyonds liquidity\', vault/src/owner.rs:45:9"
        );
    } else {
        panic!("Expected failure when payment to unregistered account.");
    }
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 9, 1200, 1000, 7800, 160);
    assert_eq!(balance_of(&token, &user3.account_id()), 1000);

}


