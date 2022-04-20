# Session Vault

## Features
Session-release for specific FT (NEP-141),
- support multiple users, up to thousands,
- support multiple round of sessions to same user,
- for each round of sessions, need (start_time, session_interval, release_per_session, session_count).

## Operation Steps

### Contract Deploy and Init

```bash
# create contract account
near create-account $VAULT --masterAccount $ROOT --initialBalance 35 --accountId $ROOT
# execute deploy
near deploy $VAULT res/session_vault.wasm --account_id=$VAULT
# initiate
near call $VAULT new '{"owner_id": "'$ROOT'", "token_id": "ref.'$FT'"}' --account_id=$VAULT
```
Note:  
- owner ID and FT contract ID are need when initiate,

### Add User
```bash
# start at 2022-04-20 09:00:00 => 1650416400
# session interval 3 month: 3 * 30 * 24 * 3600 = 7776000
near call $VAULT add_account '{"account_id": "u1.testnet", "start_timestamp": 1650416400, "session_interval": 7776000, "session_num": 4, "release_per_session": "100'$ZERO18'"}' --account_id=$ROOT --deposit=0.1
# check
near view $VAULT get_account '{"account_id": "u1.testnet"}'
```
Note:  
- Only owner can add users,
- If user is currently in a locking round, fail with ERR_ACCOUNT_IN_SESSION
- If user has NOT claimed all out from previous locking round, fail with ERR_ACCOUNT_NEED_CLAIM
- Then succeed

### Deposit Locking Token to User
```bash
near call ref.$FT ft_transfer_call '{"receiver_id": "'$VAULT'", "amount": "400'$ZERO18'", "msg": "u1.testnet"}' --account_id=anyone.testnet --depositYocto=1 --gas=100$TGAS
# check
near view $VAULT get_account '{"account_id": "u1.testnet"}'
```
Note:  
- Anyone can deposit token, msg should be the target user account ID,
- If token unmatch, fail with ERR_ILLEGAL_TOKEN
- If msg is empty, fail with ERR_MISSING_ACCOUNT_ID
- If user not exist, fail with ERR_ACCOUNT_NOT_EXIST
- If there is locked token or user has claimed all out, fail with ERR_ALREADY_DEPOSITED
- Amount should equal to session_count * release_per_session, or fail with ERR_INCORRECT_AMOUNT

### Transfer Ownership
```bash
near call $VAULT set_owner '{"owner_id": "somedao.testnet"}' --account_id=$ROOT --depositYocto=1
```
Note:  
- For production running, owner should be some DAO contract

### Claim Unlocked Token
```bash
# claim by user himself
near call $VAULT claim '' --account_id=u1.testnet --gas=100$TGAS
# claim by third-party
near call $VAULT claim '{"account_id": "u1.testnet"}' --account_id=anyone.testnet --gas=100$TGAS
```
Note:  
- If there is locking token but no unlocked amount, directly return true,
- If there is no locking token, directly return false,
- If user not exist, fail with ERR_ACCOUNT_NOT_EXIST
- If there is no token deposited, fail with ERR_NOT_ENOUGH_BALANCE
- Then contract would transfer unlocked token to user's wallet
- If transfer fails, revert claim with a log `Account claim failed and rollback, account is xxx, balance is xxx` 

### AccountInfo
```rust
pub struct AccountInfo {
    pub account_id: AccountId,

    // current round, start at timestamp in sec
    pub start_timestamp: TimestampSec,
    // current round, session interval in sec
    pub session_interval: TimestampSec,
    // current round, session count
    pub session_num: u32,
    // current round, already claimed sessions
    pub last_claim_session: u32,
    // current round, release amount per session
    pub release_per_session: WrappedBalance,

    // accumulated calimed amount
    pub claimed_amount: WrappedBalance,
    // accumulated deposited amount
    pub deposited_amount: WrappedBalance,
    // current unlocked and unclaimed amount
    pub unclaimed_amount: WrappedBalance,
}
```

### ContractInfo
```rust
pub struct ContractInfo {
    pub version: String,
    // only onwer can manage accounts
    pub owner_id: AccountId,
    // FT in locking
    pub token_account_id: AccountId,
    // accumulated deposited amount
    pub total_balance: WrappedBalance,
    // accumulated claimed amount
    pub claimed_balance: WrappedBalance,
}
```

## All Views
```rust
pub fn contract_metadata(&self) -> ContractInfo;
pub fn get_contract_storage_report(&self) -> StorageReport;
pub fn get_account(&self, account_id: ValidAccountId) -> Option<AccountInfo>;
pub fn list_accounts(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<AccountInfo>;
pub fn get_owner(&self) -> AccountId;
```

```bash
near view $VAULT contract_metadata
near view $VAULT get_contract_storage_report
near view $VAULT get_account '{"account_id": "xxx"}'
near view $VAULT list_accounts ''
```