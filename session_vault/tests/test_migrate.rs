
use near_sdk_sim::{deploy, view, init_simulator, to_yocto};

use session_vault::{ContractContract as SessionVault, ContractInfo};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PREV_SESSION_VAULT_WASM_BYTES => "../res/session_vault.wasm",
    SESSION_VAULT_WASM_BYTES => "../res/session_vault.wasm",
}

#[test]
fn test_upgrade() {
    let root = init_simulator(None);
    let test_user = root.create_user("test".to_string(), to_yocto("100"));
    let session_vault = deploy!(
        contract: SessionVault,
        contract_id: "session_vault".to_string(),
        bytes: &PREV_SESSION_VAULT_WASM_BYTES,
        signer_account: root,
        init_method: new(root.valid_account_id(), root.valid_account_id())
    );
    // Failed upgrade with no permissions.
    let result = test_user
        .call(
            session_vault.user_account.account_id.clone(),
            "upgrade",
            &SESSION_VAULT_WASM_BYTES,
            near_sdk_sim::DEFAULT_GAS,
            0,
        )
        .status();
    assert!(format!("{:?}", result).contains("ERR_NOT_ALLOWED"));

    root.call(
        session_vault.user_account.account_id.clone(),
        "upgrade",
        &SESSION_VAULT_WASM_BYTES,
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
    .assert_success();
    let metadata = view!(session_vault.contract_metadata()).unwrap_json::<ContractInfo>();
    // println!("{:#?}", metadata);
    assert_eq!(metadata.version, "1.0.0".to_string());

    // Upgrade to the same code migration is skipped.
    root.call(
        session_vault.user_account.account_id.clone(),
        "upgrade",
        &SESSION_VAULT_WASM_BYTES,
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
    .assert_success();
}