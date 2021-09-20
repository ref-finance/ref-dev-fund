use near_sdk::json_types::{U128};
use near_sdk_sim::{
    call, to_yocto, view,
};
use vault::{Stats, AccountOutput};
use crate::common::init::*;

pub mod common;


#[test]
fn three_employees() {
    println!("*** deploy and init contracts");
    let (root, owner, vault, token) = setup_vault(10000, 50, 10, 10);
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    // println!("Vault stats: {:#?}", vault_stats);
    assert_eq!(vault_stats.version, "0.3.1".to_string());
    assert_eq!(vault_stats.owner_id, owner.account_id());
    assert_eq!(vault_stats.token_account_id, token.account_id());
    assert_eq!(vault_stats.total_balance.0, 10000);
    assert_eq!(vault_stats.start_timestamp, 50);
    assert_eq!(vault_stats.release_interval, 10);
    assert_eq!(vault_stats.release_rounds, 10);
    assert_eq!(vault_stats.claimed_balance.0, 0);
    assert_eq!(vault_stats.locked_balance.0, 10000);
    assert_eq!(vault_stats.liquid_balance.0, 0);
    assert_eq!(vault_stats.unclaimed_balance.0, 0);
    assert_eq!(vault_stats.current_round, 0);


    assert!(root.borrow_runtime_mut().produce_blocks(27).is_ok());
    println!("----> Chain goes to height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 0, 0, 10000, 0, 0);

    assert!(root.borrow_runtime_mut().produce_blocks(10).is_ok());
    println!("----> Chain goes to height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 1, 0, 9000, 1000, 0);

    assert!(root.borrow_runtime_mut().produce_blocks(10).is_ok());
    println!("----> Chain goes to height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 2, 0, 8000, 2000, 0);

    println!("*** Add user1");
    let user1 = root.create_user("user1".to_string(), to_yocto("10"));
    call!(user1, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(owner, vault.add_account(user1.valid_account_id(), 50, 10, 10, U128(20))).assert_success();
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 3, 0, 7000, 3000, 60);

    println!("*** Add user2");
    let user2 = root.create_user("user2".to_string(), to_yocto("10"));
    call!(user2, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(owner, vault.add_account(user2.valid_account_id(), 55, 10, 10, U128(10))).assert_success();
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 4, 0, 6000, 4000, 110);

    println!("*** Add user3");
    let user3 = root.create_user("user3".to_string(), to_yocto("10"));
    call!(user3, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(owner, vault.add_account(user3.valid_account_id(), 60, 10, 10, U128(10))).assert_success();
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 5, 0, 5000, 5000, 180);

    println!("*** User1 claim failed cause vault has no money");
    let out_come = call!(user1, vault.claim());
    out_come.assert_success();
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("The account doesn't have enough balance"));
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 5, 0, 5000, 5000, 190);

    println!("*** Payment failed cause vault has no money");
    let out_come = call!(owner, vault.payment(user3.valid_account_id(), U128(1000)));
    out_come.assert_success();
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("The account doesn't have enough balance"));
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 6, 0, 4000, 6000, 220);

    println!("*** deposit money to vault");
    call!(
        owner,
        token.ft_transfer(vault.valid_account_id(), U128(10000), None),
        deposit = 1
    ).assert_success();
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 6, 0, 4000, 6000, 220);

    assert!(root.borrow_runtime_mut().produce_blocks(7).is_ok());
    println!("----> Chain goes to height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 7, 0, 3000, 7000, 260);

    println!("*** User1 claim OK");
    let out_come = call!(user1, vault.claim());
    out_come.assert_success();
    assert_eq!(out_come.promise_errors().len(), 0);
    assert_eq!(balance_of(&token, &user1.account_id()), 140);
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 7, 140, 3000, 6860, 130);

    println!("*** Payment failed cause exceeds liquidity");
    let out_come = call!(owner, vault.payment(user3.valid_account_id(), U128(6800)), deposit = 0);
    assert!(!out_come.is_ok());
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("The payment amount beyonds liquidity"));
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 7, 140, 3000, 6860, 130);

    println!("*** Payment OK");
    let out_come = call!(owner, vault.payment(user3.valid_account_id(), U128(1000)));
    out_come.assert_success();
    assert_eq!(out_come.promise_errors().len(), 0);
    assert_eq!(balance_of(&token, &user3.account_id()), 1000);
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 8, 1140, 2000, 6860, 160);

    println!("*** Payment failed with user3 not register to token");
    call!(user3, token.storage_unregister(Some(true)), deposit = 1).assert_success();
    let out_come = call!(owner, vault.payment(user3.valid_account_id(), U128(1000)));
    out_come.assert_success();
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("The account user3 is not registered"));
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 9, 1140, 1000, 7860, 200);

    println!("*** User3 claim failed with user3 not register to token");
    let out_come = call!(user3, vault.claim());
    out_come.assert_success();
    let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
    assert!(ex_status.contains("The account user3 is not registered"));
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 9, 1140, 1000, 7860, 210);

    println!("*** Remove User3");
    let user_info = view!(vault.get_account(user3.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 0, 80);
    let out_come = call!(owner, vault.remove_account(user3.valid_account_id()), deposit = 0);
    out_come.assert_success();
    assert_eq!(out_come.promise_errors().len(), 0);
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 10, 1140, 0, 8860, 150);

    println!("*** Remove User1");
    let user_info = view!(vault.get_account(user1.valid_account_id())).unwrap_json::<AccountOutput>();
    assert_userinfo(&user_info, 7, 60);
    let out_come = call!(owner, vault.remove_account(user1.valid_account_id()), deposit = 0);
    out_come.assert_success();
    assert_eq!(out_come.promise_errors().len(), 0);
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 10, 1140, 0, 8860, 90);

    assert!(root.borrow_runtime_mut().produce_blocks(17).is_ok());
    println!("----> Chain goes to height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 12, 1140, 0, 8860, 100);

    println!("*** User2 claim OK");
    let out_come = call!(user2, vault.claim());
    out_come.assert_success();
    assert_eq!(out_come.promise_errors().len(), 0);
    assert_eq!(balance_of(&token, &user2.account_id()), 100);
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 12, 1240, 0, 8760, 0);

    assert!(root.borrow_runtime_mut().produce_blocks(15).is_ok());
    println!("----> Chain goes to height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 14, 1240, 0, 8760, 0);

    println!("*** User2 claim nothing");
    let out_come = call!(user2, vault.claim());
    out_come.assert_success();
    assert_eq!(out_come.promise_errors().len(), 0);
    assert_eq!(balance_of(&token, &user2.account_id()), 100);
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 14, 1240, 0, 8760, 0);

    println!("*** Payment all");
    let out_come = call!(owner, vault.payment(user1.valid_account_id(), U128(8760)));
    out_come.assert_success();
    assert_eq!(out_come.promise_errors().len(), 0);
    assert_eq!(balance_of(&token, &user1.account_id()), 8900);
    assert_eq!(balance_of(&token, &vault.account_id()), 0);
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    assert_stats(&vault_stats, 14, 10000, 0, 0, 0);
}