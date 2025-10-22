# ICM_PROGRAM AUDIT BY [totdking](x.com/totdking)
## Description of the protocol

The ICM is ....

### Severity classification
- Critical: This causes full drainage of user funds, clears protocol of total value locked
- High: This causes significant damage to the protocol or users and the cost to perform is negligible to damage incurred.
- Medium: This costs minimal damage to protocol or may not be incentivized to execute.
- Low: Unexpected behaviour occurs here or negligible loss or attack surface that depends on so many unlikely factors.
- Informational: This is negligible and is mostly better design or better coding practices applied

## Summary
Found bugs are 3 Highs, 1 medium and 2 informationals

|ID | Title | Severity |
|---|---|---|
| [H-01](#h-01-usdc-mint--token-mint-is-not-specified-leading-to-any-arbitrary-mint-address-set) | Token mint address is not constrained | High |
| [H-02](#h-02-improper-closing-of-several-data-and-token-accounts-alike) | Improper closing of accounts | High |
| [H-03](#h-03-bucket-account-in-close_bucket-was-not-declared-as-mutable-invalidating-all-state-change--write-operations-to-that-account) | Invalid declaration of bucket in close_bucket | High |
| [M-01](#m-01-token-accounts-that-were-initialized-were-not-closed-accordingly) | Token Accounts initialized had no close mechanism | Medium ||  |  |  |
| [IN-01](#in-01--manual-calculation-of-account-space-of-data-pda-accounts)| Manual calculation of account space | Informational |
| [IN-02](#in-02-redundant-bucket-name-parameter-in-instruction-contexts)| Manual passing of bucket name each time as instruction | Informational |
|  |  |  |
|  |  |  |
|  |  |  |


## [H-01] USDC MINT / Token mint is not specified leading to any arbitrary mint address set

### Description:
In the [create_bucket](../programs/icm-program/src/instructions/create_bucket.rs), the initialization of the usdc mint in the initiaize_bucket_handler can accept any value of the usdc mint 
### Root Cause
`
pub usdc_mint: Box<Account<'info, Mint>>,
`

This can accept any address especially an address that can mint worthless tokens and in a pool where users swap real sol for the said tokens in the bucket, instead of getting tokens with worth, their original funds will be siphoned off and the sol is locked in the vault.

### Impact
If they try to swap or withdraw the tokens they got from the usdc bad minter, the jup agg. will flag it everytime as insufficient funds due to the worthlessness of the tokens.

### Remediation
Use the anchor account constraint to ensure the right mint address is passed in to the bucket

`
#[account(address = usdc_id() @ ErrorCode::InvalidMint)]
pub usdc_mint: Box<Account<'info, Mint>>,
`

## [H-02] Improper closing of several data and token accounts alike

### Description
In the close bucket, we have the logic to close all account after trading is completed or the user wants to opt out after withdrawal from the icm program.

What was there initially was using the close_bucket util which jsut updated state but did not transfer any money out either to the bucket creator or the trading pool.

### Root Cause
In this commit [close_bucket](https://github.com/OkarFabianTheWise/icm-program/commit/9689dad5c3107c31ab65041a853bd3a9f6aa1bc5#diff-d3558e0f2cabe94a9368c2799c63dd9dcf8c1d47b107fbe7a7295f6bed0a9cc2). The only change was updating the successfully managed volume without actually closing the used accounts. 

### Impact
This can lead to reinitialization attacks where accounts not supposed to be in use come in use and does not let rent of said accounts be transferred

### Remediation
Make use of the close anchor helper
```rust
#[account(
        mut,
        close = creator,
        seeds = [b"bucket", bucket.name.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub bucket: Box<Account<'info, Bucket>>,

#[account(
    mut,
    close = creator,
    seeds = [b"trading_pool", bucket.name.as_bytes(), creator.key().as_ref()],
    bump
)]
pub trading_pool: Account<'info, crate::state::TradingPool>,
```

## [H-03] Bucket account in claim-rewards was not declared as mutable invalidating all state change / write operations to that account

### Description
In the claim_rewards.rs, the bucket was declared as an immutable account without the `mut` keyword, and was later used as a borrowed mutable in the close_bucket_util.

### Root Cause
Lack of mut keyword in the declaration of the bucket account in the claim_rewards.

### Impact
1. State changes fail
2. Runtime error even if it compiles successfully

### Remediation
Add the mut keyword to the bucket parameter definition


## [M-01] Token accounts that were initialized were not closed accordingly

### Description
In the close bucket, the closing of token accounts was not present as they were explicitly initialized in the contribute_to_bucket module

### Root Cause
Lack of closing mechanism

### Impact
This can lead to dangling accounts which the rent would be left in the account just dangling also this can lead to increase in bloat of cu being consumed each time the program is invoked, raising gas prices

### Remediation
What was implemented was the CloseAccount instruction from the anchor_spl crate, this is a manual process but does the same work as the `close = creator` constraint in [H-02](#remediation-h-02-improper-closing-of-several-data-and-token-accounts-alike) remediation


## [IN-01]  Manual calculation of Account space of Data pda accounts

### Description
In the codebase, there are incosistent calculation of space needed by accounts to occupy, In which sometimes can be prone to error if the account struct has a lot of fields.

### Impact
Wrong calculation of space used by the program

### Remediation
Use the `INIT_SPACE` account macro provided by anchor to automatically calculate account space

`
#[account]
#[derive(InitSpace)]
pub struct Bucket {
    //Rest of code
}
`


## [IN-02] Redundant bucket name parameter in instruction contexts

### Description
Multiple instruction contexts (`contribute_to_bucket`, `close_bucket`, `swap_tokens`, etc.) require a `bucket_name` parameter to be passed as an instruction argument. This is redundant since the bucket name is already stored in the `Bucket` account and is immutable after initialization.

### Root Cause
The current implementation requires clients to pass the bucket name as an instruction parameter even though this information is already available in the on-chain `Bucket` account data.

### Impact
- **Increased transaction size**: Unnecessary data increases transaction costs
- **Client complexity**: Clients must maintain and pass bucket names for every interaction
- **Potential inconsistencies**: Risk of passing incorrect bucket names that don't match the actual bucket
- **Poor developer experience**: Additional parameter management burden

### Remediation
Remove the `bucket_name` parameter from instruction contexts and derive the bucket name from the `Bucket` account data instead. Since the bucket name is immutable after creation, this approach is both more efficient and eliminates potential mismatches.

**Example improvement:**
```rust
// Instead of requiring bucket_name as parameter
pub fn contribute_to_bucket_handler(
    ctx: Context<ContributeToBucket>,
    bucket_name: String, // Remove this
    amount: u64,
) -> Result<()>

// Use bucket.name directly from the account
let bucket_name = &ctx.accounts.bucket.name;
```

And in seed derivation 
```rust
#[derive(Accounts)]
// #[instruction(bucket_name: String)] <- Remove this and use the bucket.name.as_bytes() instead
pub struct CloseBucket<'info> {
    #[account(
        mut,
        close = creator,
        seeds = [b"bucket", bucket.name.as_bytes(), creator.key().as_ref()],
        bump
    )]
    pub bucket: Box<Account<'info, Bucket>>,
}
```