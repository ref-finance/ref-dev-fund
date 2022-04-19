use near_sdk::json_types::{U128};
use near_sdk_sim::{
    call, to_yocto, view,
};
use crate::common::{
    init::*,
    types::*,
    utils::*,
};

pub mod common;

#[test]
fn sim_set_owner() {
    let (root, owner, session_vault, _) = setup_vault();
    let user1 = root.create_user("user1".to_string(), to_yocto("10"));

    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.owner_id, owner.account_id());

    let out_come =call!(
        user1,
        session_vault.set_owner(user1.valid_account_id()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_ALLOWED"));

    call!(
        owner,
        session_vault.set_owner(user1.valid_account_id()),
        deposit = 1
    ).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.owner_id, user1.account_id());

    call!(
        user1,
        session_vault.add_account(user1.valid_account_id(), 10, 10, 1, 100.into()),
        deposit = to_yocto("0.1")
    ).assert_success();


    call!(
        user1,
        session_vault.set_owner(owner.valid_account_id()),
        deposit = 1
    ).assert_success();

    let out_come =call!(
        user1,
        session_vault.add_account(user1.valid_account_id(), 10, 10, 1, 100.into()),
        deposit = to_yocto("0.1")
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_ALLOWED"));
}

#[test]
fn sim_add_user() {
    let (root, owner, session_vault, token) = setup_vault();
    root.borrow_runtime_mut().cur_block.block_timestamp = 0;

    let user1 = root.create_user("user1".to_string(), to_yocto("10"));
    call!(user1, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();

    let out_come =call!(
        user1,
        session_vault.add_account(user1.valid_account_id(), 10, 10, 1, 100.into()),
        deposit = to_yocto("0.1")
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_ALLOWED"));

    call!(
        owner,
        session_vault.add_account(user1.valid_account_id(), 10, 10, 1, 100.into()),
        deposit = to_yocto("0.1")
    ).assert_success();

    let out_come =call!(
        owner,
        session_vault.add_account(user1.valid_account_id(), 10, 10, 1, 100.into()), 
        deposit = to_yocto("0.1")
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ACCOUNT_IN_SESSION"));

    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(20);
    let out_come =call!(
        owner,
        session_vault.add_account(user1.valid_account_id(), 10, 10, 1, 100.into()),
        deposit = to_yocto("0.1")
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ACCOUNT_NEED_CLAIM"));
}

#[test]
fn sim_deposit_token() {
    let (root, owner, session_vault, token) = setup_vault();
    let other_token = test_token(&root, "other_token".to_string(), vec![session_vault.account_id(), owner.account_id()]);
    call!(owner, other_token.mint(U128(10000))).assert_success();
    root.borrow_runtime_mut().cur_block.block_timestamp = 0;

    let user1 = root.create_user("user1".to_string(), to_yocto("10"));
    call!(user1, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();

    call!(
        owner,
        session_vault.add_account(user1.valid_account_id(), 10, 10, 1, 100.into()),
        deposit = to_yocto("0.1")
    ).assert_success();

    let out_come = call!(
        owner,
        other_token.ft_transfer_call(session_vault.valid_account_id(), U128(100), None, user1.account_id()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ILLEGAL_TOKEN"));

    let out_come = call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(100), None, "".to_string()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_MISSING_ACCOUNT_ID"));

    let out_come = call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(100), None, "user2".to_string()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ACCOUNT_NOT_EXIST"));

    let out_come = call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(110), None, "user1".to_string()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_INCORRECT_AMOUNT"));

    call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(100), None, user1.account_id()),
        deposit = 1
    ).assert_success();

    let out_come = call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(100), None, "user1".to_string()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ALREADY_DEPOSITED"));

    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(20);
    call!(
        user1,
        session_vault.claim(None)
    ).assert_success();
    let user_info = view!(session_vault.get_account(user1.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.unclaimed_amount.0, 0);
    let out_come = call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(100), None, "user1".to_string()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ALREADY_DEPOSITED"));
}

#[test]
fn sim_claim() {
    let (root, owner, session_vault, token) = setup_vault();
    // println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);
    
    root.borrow_runtime_mut().cur_block.block_timestamp = 0;

    let user1 = root.create_user("user1".to_string(), to_yocto("10"));
    
    call!(
        owner,
        session_vault.add_account(user1.valid_account_id(), 10, 10, 1, 100.into()),
        deposit = to_yocto("0.1")
    ).assert_success();

    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 0);
    assert_eq!(contract_info.total_balance.0, 0);

    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(20);

    let user_info = view!(session_vault.get_account(user1.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.unclaimed_amount.0, 100);

    let out_come = call!(
        owner,
        session_vault.claim(Some(owner.valid_account_id()))
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_ACCOUNT_NOT_EXIST"));

    let out_come = call!(
        owner,
        session_vault.claim(Some(user1.valid_account_id()))
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_ENOUGH_BALANCE"));

    call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(100), None, user1.account_id()),
        deposit = 1
    ).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 0);
    assert_eq!(contract_info.total_balance.0, 100);

    let out_come = call!(
        user1,
        session_vault.claim(None)
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("The account user1 is not registered"));
    println!("{:?}", get_logs(&out_come));
    let user_info = view!(session_vault.get_account(user1.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.unclaimed_amount.0, 100);

    call!(user1, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();

    call!(
        user1,
        session_vault.claim(None)
    ).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 100);
    assert_eq!(contract_info.total_balance.0, 100);
    let user_info = view!(session_vault.get_account(user1.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(100, balance_of(&token, &user1.account_id()));

    call!(
        owner,
        session_vault.add_account(user1.valid_account_id(), 20, 20, 2, 100.into()),
        deposit = to_yocto("0.1")
    ).assert_success();

    let out_come = call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(100), None, user1.account_id()),
        deposit = 1
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_INCORRECT_AMOUNT"));

    call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(200), None, user1.account_id()),
        deposit = 1
    ).assert_success();

    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(40);

    call!(
        user1,
        session_vault.claim(None)
    ).assert_success();

    assert_eq!(200, balance_of(&token, &user1.account_id()));

    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(60);

    call!(
        owner,
        session_vault.claim(Some(user1.valid_account_id()))
    ).assert_success();

    assert_eq!(300, balance_of(&token, &user1.account_id()));
}
