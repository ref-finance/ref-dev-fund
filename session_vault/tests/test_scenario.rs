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
fn sim_one_round() {
    let (root, owner, session_vault, token) = setup_vault();
    
    root.borrow_runtime_mut().cur_block.block_timestamp = 0;

    let alice = root.create_user("alice".to_string(), to_yocto("10"));
    let bob = root.create_user("bob".to_string(), to_yocto("10"));
    let charlie = root.create_user("charlie".to_string(), to_yocto("10"));

    call!(alice, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(bob, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();
    call!(charlie, token.storage_deposit(None, None), deposit = to_yocto("1")).assert_success();

    // start from 100 sec, and release 100 token per 100 sec for 4 times, so the end is 500 sec.
    call!(owner, session_vault.add_account(alice.valid_account_id(), 100, 100, 4, 100.into()), deposit = to_yocto("0.1")).assert_success();
    call!(owner, session_vault.add_account(bob.valid_account_id(), 100, 100, 4, 100.into()), deposit = to_yocto("0.1")).assert_success();
    call!(owner, session_vault.add_account(charlie.valid_account_id(), 100, 100, 4, 100.into()), deposit = to_yocto("0.1")).assert_success();

    // fill tokens
    call!(owner, token.ft_transfer_call(session_vault.valid_account_id(), U128(400), None, alice.account_id()), deposit = 1).assert_success();
    call!(owner, token.ft_transfer_call(session_vault.valid_account_id(), U128(400), None, bob.account_id()), deposit = 1).assert_success();
    call!(owner, token.ft_transfer_call(session_vault.valid_account_id(), U128(400), None, charlie.account_id()), deposit = 1).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 0);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 0);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 0);
    assert_eq!(0, balance_of(&token, &alice.account_id()));
    // and claim would got nothing changed
    call!(alice, session_vault.claim(None)).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 0);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 0);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 0);
    assert_eq!(0, balance_of(&token, &alice.account_id()));

    // go to the start time
    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(100);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 0);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 0);
    assert_eq!(0, balance_of(&token, &alice.account_id()));
    // and claim would got nothing changed
    call!(alice, session_vault.claim(None)).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 0);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 0);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 0);
    assert_eq!(0, balance_of(&token, &alice.account_id()));

    // pass one interval
    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(200);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 0);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 100);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 0);
    assert_eq!(0, balance_of(&token, &alice.account_id()));
    // and claim does something
    call!(alice, session_vault.claim(None)).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 100);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 100);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 1);
    assert_eq!(100, balance_of(&token, &alice.account_id()));

    // go to 1.5 interval
    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(250);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 100);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 1);
    assert_eq!(100, balance_of(&token, &alice.account_id()));
    // and claim does nothing
    call!(alice, session_vault.claim(None)).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 100);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 100);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 1);
    assert_eq!(100, balance_of(&token, &alice.account_id()));

    // go to 2 interval
    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(300);
    let user_info = view!(session_vault.get_account(bob.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 0);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 200);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 0);
    assert_eq!(0, balance_of(&token, &bob.account_id()));
    // and claim 2 sessions
    call!(owner, session_vault.claim(Some(bob.valid_account_id()))).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 300);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(bob.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 200);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 2);
    assert_eq!(200, balance_of(&token, &bob.account_id()));
    // and claim again does nothing
    call!(bob, session_vault.claim(None)).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 300);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(bob.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 200);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 2);
    assert_eq!(200, balance_of(&token, &bob.account_id()));

    // go to 4 interval
    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(500);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 100);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 300);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 1);
    assert_eq!(100, balance_of(&token, &alice.account_id()));
    // and claim does something
    call!(owner, session_vault.claim(Some(alice.valid_account_id()))).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 600);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 400);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 4);
    assert_eq!(400, balance_of(&token, &alice.account_id()));
    // and claim again does nothing
    call!(alice, session_vault.claim(None)).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 600);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(alice.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 400);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 4);
    assert_eq!(400, balance_of(&token, &alice.account_id()));

    // go to 5 interval
    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(600);
    let user_info = view!(session_vault.get_account(charlie.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 0);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 400);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 0);
    assert_eq!(0, balance_of(&token, &charlie.account_id()));
    // and claim does something
    call!(owner, session_vault.claim(Some(charlie.valid_account_id()))).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 1000);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(charlie.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 400);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 4);
    assert_eq!(400, balance_of(&token, &charlie.account_id()));
    // and claim again does nothing
    call!(charlie, session_vault.claim(None)).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 1000);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(charlie.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 400);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 4);
    assert_eq!(400, balance_of(&token, &charlie.account_id()));

    // go to 6 interval, end everything
    root.borrow_runtime_mut().cur_block.block_timestamp = to_nano(700);
    let user_info = view!(session_vault.get_account(bob.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.unclaimed_amount.0, 200);
    call!(owner, session_vault.claim(Some(alice.valid_account_id()))).assert_success();
    call!(owner, session_vault.claim(Some(bob.valid_account_id()))).assert_success();
    call!(owner, session_vault.claim(Some(charlie.valid_account_id()))).assert_success();
    let contract_info = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    assert_eq!(contract_info.claimed_balance.0, 1200);
    assert_eq!(contract_info.total_balance.0, 1200);
    let user_info = view!(session_vault.get_account(bob.valid_account_id())).unwrap_json::<AccountInfo>();
    assert_eq!(user_info.claimed_amount.0, 400);
    assert_eq!(user_info.deposited_amount.0, 400);
    assert_eq!(user_info.unclaimed_amount.0, 0);
    assert_eq!(user_info.start_timestamp, 100);
    assert_eq!(user_info.session_interval, 100);
    assert_eq!(user_info.session_num, 4);
    assert_eq!(user_info.release_per_session.0, 100);
    assert_eq!(user_info.last_claim_session, 4);
    assert_eq!(400, balance_of(&token, &bob.account_id()));
}