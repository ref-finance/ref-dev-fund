# ref-dev-fund

## suggest steps of using the vault
* deploy vault
* prepare a multisig account
* initialize vault with releasing rule and owner set to that multisig account
* owner add users
* transfer token that need be managed by the vault 

## core logic
The vault contract is responsible for keeping and releasing whole dev funds.  
There are two pools in this vault:  
* locking pool, holds all token that still in locking;
* liquidity pool, holds all token that has been release but undistribute yet.  
Here, distribution includes two type of actions:
  1. claim action by users, to handle regular salary;
  2. payment action by owner, to handle temp expenditure, like office expenses, part-time job reward and etc.

the vault global info can be learned from this view fucntion:
```rust
pub fn get_stats(&self) -> Stats;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct Stats {
    pub version: String,
    // only onwer can manage accounts and call payments
    pub owner_id: AccountId,
    // token keeped by this vault
    pub token_account_id: AccountId,
    // the static total balance in this vault
    pub total_balance: WrappedBalance,
    // the start point of linear release 
    pub start_timestamp: TimestampSec,
    // the duration of each release round
    pub release_interval: TimestampSec,
    // the total release rounds, 
    // we can infer release_per_round = total_balance / release_rounds
    pub release_rounds: u32,
    // already claimed balance, includes account claims and payments
    pub claimed_balance: WrappedBalance,
    // following are calculated from current env
    pub locked_balance: WrappedBalance,  // still locked in this vault
    pub liquid_balance: WrappedBalance,  // liquid balance in this vault
    pub unclaimed_balance: WrappedBalance,  // can be claimed for current
    pub current_round: u32,  // the current release round, start from 1
}
```

There are two steps in this vault logic:
* global release, that makes tokens in locking pool flow to liquidity pool as planed;
* user release, that manages each users balance when user call `claim` and payments called by owner.

## initialize

When initializing, we need set:  
* `total_balance`: the total token asset kept in this vault;
* `start_timestamp`: linux timestamp (in sec) when the releasing starts;
* `release_interval`: in sec, the linear release is split into rounds;
* `release_rounds`: so `release_amount_per_round = total_balance / release_rounds`

```rust
pub fn new(
    owner_id: ValidAccountId,
    token_account_id: ValidAccountId,

    total_balance: WrappedBalance,
    start_timestamp: TimestampSec,
    release_interval: TimestampSec,
    release_rounds: u32,
)
```

## owner methods

### change owner
```rust
pub fn set_owner(&mut self, owner_id: ValidAccountId);
```

### add user

For each user, instead of setting `total_balance` as we do in global one, we need to set `release_per_round`. The reason of this choice is to align with situation in real world where monthly or yearly salary is more common than a total balance of salary.

```rust
pub fn add_account(
    &mut self, 
    account_id: ValidAccountId,
    start_timestamp: TimestampSec,
    release_interval: TimestampSec,
    release_rounds: u32,
    release_per_round: WrappedBalance,
) -> bool;
```

Each user has an account structure in contract record user state. It can be learned from this view function:
```rust
pub fn get_account(&self, account_id: ValidAccountId) -> Option<AccountOutput>

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct AccountOutput {
    pub account_id: AccountId,
    // the linear release start time point for this account
    pub start_timestamp: TimestampSec,
    // the duration of each claim round for this account
    pub release_interval: TimestampSec,
    // the total rounds of release
    pub release_rounds: u32,
    // the round index of previous claim, start from 1
    pub last_claim_round: u32,
    // total_release = release_rounds * release_per_round
    pub release_per_round: WrappedBalance,
    // unclaimed amount
    pub unclaimed_amount: WrappedBalance,
}
```
### remove user
Onwer has power to remove user to support halfway left of users.
```rust
pub fn remove_account(&mut self, account_id: ValidAccountId) -> bool;
```

### payment

```rust
pub fn payment(&mut self, receiver_id: ValidAccountId, amount: WrappedBalance) -> PromiseOrValue<bool>;
```
Notes: 
1. The vault would assert the liquidity can support this payment with consideration of unclaimed balance of all users.
2. If token transfer fails, such as unregister of receiver in token contract and etc, the payment would roll back to ensure data integrity.

## user functions

Using `get_account` to get current state, user can call claim to get their salary back to their own wallet. Remember to register himself to the token contract before claiming. Again, do not worry if you forget to register, like payment, claim would rollback state when transfer fails in any reason.

```rust
pub fn claim(&mut self) -> PromiseOrValue<bool>;
```
