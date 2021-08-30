use near_sdk::json_types::{U128};
use near_sdk_sim::{
    call, to_yocto, view,
};
use vault::{Stats, AccountOutput};
use crate::common::init::*;

mod common;


#[test]
fn core_logic() {
    println!("*** deploy and init contracts");
    let (root, owner, vault, token) = setup_vault(10000, 50, 10, 10);
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
    call!(owner, vault.add_account(user1.valid_account_id(), 50, 10, 10, U128(20)), deposit = 0)
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
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 4, 20);

    println!("*** Add user2 without storage_deposit to token");
    let user2 = root.create_user("user2".to_string(), to_yocto("10"));
    call!(owner, vault.add_account(user2.valid_account_id(), 50, 10, 10, U128(20)), deposit = 0)
    .assert_success();
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 5, 80, 5000, 4920, 120);

    println!("*** User2 try to claim but fail");
    let out_come = call!(user2, vault.claim(), deposit = 0);
    out_come.assert_success();
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    // println!("ex_status: {}", ex_status);
    assert!(ex_status.contains("The account user2 is not registered"));
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 6, 80, 4000, 5920, 160);
    assert_eq!(balance_of(&token, &user2.account_id()), 0);
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 4, 40);
    let user_info = view!(vault.get_account(user2.valid_account_id())).unwrap_json::<AccountOutput>();
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
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 4, 60);
    let user_info = view!(vault.get_account(user2.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 6, 20);

    println!("*** Payment 1000 to user3 but without storage_deposit to token");
    let user3 = root.create_user("user3".to_string(), to_yocto("10"));
    let out_come = call!(owner, vault.payment(to_va(user3.account_id()), U128(1000)), deposit = 0);
    out_come.assert_success();
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    // println!("ex_status: {}", ex_status);
    assert!(ex_status.contains("The account user3 is not registered"));
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
    let out_come = call!(owner, vault.payment(to_va(user3.account_id()), U128(6800)), deposit = 0);
    assert!(!out_come.is_ok());
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    // println!("ex_status: {}", ex_status);
    assert!(ex_status.contains("The payment amount beyonds liquidity"));
    println!("block env ----> height: {}, ts: {}", root.borrow_runtime().current_block().block_height, root.borrow_runtime().current_block().block_timestamp);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 9, 1200, 1000, 7800, 160);
    assert_eq!(balance_of(&token, &user3.account_id()), 1000);
}

#[test]
fn remove_user() {
    let (root, owner, vault, token) = setup_vault(10000, 50, 10, 10);

    let user1 = root.create_user("user1".to_string(), to_yocto("10"));
    let user2 = root.create_user("user2".to_string(), to_yocto("10"));
    call!(user1, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(user2, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(owner, vault.add_account(user1.valid_account_id(), 50, 10, 10, U128(20)), deposit = 0)
    .assert_success();
    call!(owner, vault.add_account(user2.valid_account_id(), 50, 10, 10, U128(20)), deposit = 0)
    .assert_success();

    assert!(root.borrow_runtime_mut().produce_blocks(36).is_ok());
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 3, 0, 7000, 3000, 120);
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 0, 60);
    let user_info = view!(vault.get_account(user2.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 0, 60);

    call!(user1, vault.claim(), deposit = 0).assert_success();
    call!(user2, vault.claim(), deposit = 0).assert_success();
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 4, 120, 6000, 3880, 40);
    assert_eq!(balance_of(&token, &user1.account_id()), 60);
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 3, 20);
    let user_info = view!(vault.get_account(user2.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 3, 20);

    assert!(root.borrow_runtime_mut().produce_blocks(20).is_ok());
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 6, 120, 4000, 5880, 120);
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 3, 60);
    let user_info = view!(vault.get_account(user2.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 3, 60);
    call!(owner, vault.remove_account(user2.valid_account_id()), deposit = 0).assert_success();
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 3, 60);
    let user_info = view!(vault.get_account(user2.valid_account_id())).unwrap_json::<Option<AccountOutput>>();
    assert!(user_info.is_none());
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 6, 120, 4000, 5880, 60);

    let out_come = call!(user2, vault.claim(), deposit = 0);
    assert!(!out_come.is_ok());
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("Account not exist in this contract"));
}

#[test]
fn life_cycle() {
    let (root, owner, vault, token) = setup_vault(100, 50, 10, 10);

    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 3, 0, 70, 30, 0);

    let user1 = root.create_user("user1".to_string(), to_yocto("10"));
    call!(user1, token.storage_deposit(None, None), deposit = to_yocto("1"))
    .assert_success();
    call!(owner, vault.add_account(user1.valid_account_id(), 50, 10, 10, U128(2)), deposit = 0)
    .assert_success();
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 4, 0, 60, 40, 8);

    // round 4, claim, round 5
    call!(user1, vault.claim(), deposit = 0).assert_success();
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 5, 8, 50, 42, 2);
    assert_eq!(balance_of(&token, &user1.account_id()), 8);
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 4, 2);

    // go to round 11
    assert!(root.borrow_runtime_mut().produce_blocks(60).is_ok());
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 11, 8, 0, 92, 12);
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 4, 12);

    // go to round 13, and claim
    assert!(root.borrow_runtime_mut().produce_blocks(20).is_ok());
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 13, 8, 0, 92, 12);
    call!(user1, vault.claim(), deposit = 0).assert_success();
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 13, 20, 0, 80, 0);
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 10, 0);

    // go to round 15, and claim
    assert!(root.borrow_runtime_mut().produce_blocks(20).is_ok());
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 15, 20, 0, 80, 0);
    call!(user1, vault.claim(), deposit = 0).assert_success();
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 15, 20, 0, 80, 0);
    assert_eq!(balance_of(&token, &user1.account_id()), 20);
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 10, 0);
}
