# Session Vault

### Requirements
Distribute Token into users with locking for several sessions. When the locking expire, user can claim all tokens in one shot.

### Interfaces

#### Owner part

```rust
pub fn set_owner(&mut self, owner_id: ValidAccountId);

/// panic if account_id has active locking
pub fn add_account(
    &mut self, 
    account_id: ValidAccountId,
    start_timestamp: TimestampSec,
    session_interval: TimestampSec,
    session_num: u32,
    release_per_round: WrappedBalance,
);

```

#### Contributor part
```rust
/// ft_transfer_call with msg user_account_id
/// record token amount into user's inner structure.
```

#### User part

```rust
pub struct ContractInfo {
    pub version: String,
    // only onwer can manage accounts
    pub owner_id: AccountId,
    // token kept by this vault
    pub token_account_id: AccountId,
    // the total realized amount in this vault
    pub total_balance: WrappedBalance,
    // already claimed balance
    pub claimed_balance: WrappedBalance,
}
pub fn contract_metadata(&self) -> ContractInfo;

pub struct AccountInfo {
    pub account_id: AccountId,
    // session start time
    pub start_timestamp: TimestampSec,
    // per session lasts, eg: 90 days 
    pub session_interval: TimestampSec,
    // totally how many session, eg: 1
    pub session_num: u32,
    // the session index of previous claim, start from 1
    pub last_claim_session: u32,
    // expected total_amount = session_num * release_per_session
    pub release_per_session: WrappedBalance,
    // actually deposited amount for the user
    // each time ft_transfer_call would increase this one
    // and realized_total_amount should >= expected total_amount to make it valid
    pub realized_total_amount: WrappedBalance,
    // unclaimed amount
    pub unclaimed_amount: WrappedBalance,
}

pub fn get_account(&self, account_id: ValidAccountId) -> Option<AccountInfo>;

/// user claim his token.
/// false if user has claimed all
/// true if nothing can claim
/// panic if user not found
/// transfer_token<promise>, if there is some token can be claimed.
pub fn claim(&mut self) -> PromiseOrValue<bool>;
```