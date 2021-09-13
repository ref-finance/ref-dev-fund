use near_sdk::json_types::{U128};
use near_sdk_sim::{call, deploy, view, init_simulator, to_yocto};
use vault::{ContractContract as Vault, Stats};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PREV_VAULT_WASM_BYTES => "../res/vault_v030.wasm",
    VAULT_WASM_BYTES => "../res/vault_release.wasm",
}


use crate::common::init::*;
pub mod common;

#[test]
fn test_upgrade() {
    let root = init_simulator(None);
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let vault = deploy!(
        contract: Vault,
        contract_id: "vault".to_string(),
        bytes: &PREV_VAULT_WASM_BYTES,
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
    let token = test_token(&root, "test_token".to_string(), vec!["vault".to_string(), owner.account_id()]);

    call!(owner, token.mint(U128(10000))).assert_success();

    call!(
        owner,
        token.ft_transfer(to_va("vault".to_string()), U128(10000), None),
        deposit = 1
    ).assert_success();


    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    println!("{}", vault_stats.version);

    // Failed upgrade with no permissions.
    let result = root
        .call(
            vault.user_account.account_id.clone(),
            "upgrade",
            &VAULT_WASM_BYTES,
            near_sdk_sim::DEFAULT_GAS,
            0,
        )
        .status();
    // println!("{:#?}", result);
    assert!(format!("{:?}", result).contains("ERR_NOT_ALLOWED"));
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    println!("{}", vault_stats.version);

    owner.call(
        vault.user_account.account_id.clone(),
        "upgrade",
        &VAULT_WASM_BYTES,
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
    .assert_success();
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    println!("{}", vault_stats.version);

    // Upgrade to the same code migration is skipped.
    owner.call(
        vault.user_account.account_id.clone(),
        "upgrade",
        &VAULT_WASM_BYTES,
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
    .assert_success();
    let vault_stats = view!(vault.get_stats()).unwrap_json::<Stats>();
    println!("{}", vault_stats.version);

}