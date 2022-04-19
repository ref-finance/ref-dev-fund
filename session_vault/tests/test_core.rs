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
fn core_logic() {
    println!("*** deploy and init contracts");
    let (root, owner, session_vault, token) = setup_vault();
    println!("block env ----> height: {}", root.borrow_runtime().current_block().block_height);

    
    root.borrow_runtime_mut().cur_block.block_timestamp = 0;

    let user1 = root.create_user("user1".to_string(), to_yocto("10"));
    call!(user1, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(
        owner,
        session_vault.add_account(user1.valid_account_id(), 10, 10, 1, 100.into())
    ).assert_success();

    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    let user1_info = view!(session_vault.get_account(user1.valid_account_id())).unwrap_json::<AccountInfo>();
    println!("{:?}", contract_info);
    println!("{:?}", user1_info);

    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(20);

    let out_come = call!(
        user1,
        session_vault.claim(None)
    );
    assert_eq!(get_error_count(&out_come), 1);
    assert!(get_error_status(&out_come).contains("ERR_NOT_ENOUGH_BALANCE"));

    call!(
        owner,
        token.ft_transfer_call(session_vault.valid_account_id(), U128(100), None, user1.account_id()),
        deposit = 1
    ).assert_success();

    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    let user1_info = view!(session_vault.get_account(user1.valid_account_id())).unwrap_json::<AccountInfo>();
    println!("{:?}", contract_info);
    println!("{:?}", user1_info);

    call!(
        user1,
        session_vault.claim(None)
    ).assert_success();

    assert_eq!(100, balance_of(&token, &user1.account_id()));

    call!(
        owner,
        session_vault.add_account(user1.valid_account_id(), 20, 20, 2, 100.into())
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
