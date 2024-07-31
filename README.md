# ADDIX+FOMO validator node rewards distribution blueprint

## This blueprint has the following characteristics
* Users have to register anonymously with their Radix wallet to get an NFT. A NewUserNftEvent is emitted when this happens.
* User badges are soulbound.
* The owner can deposit the rewards ahead of time (rug proof), before actually assigning them to users. Different rewards are allowed.
* The presence of a scheduled offchain operation that takes a snapshot of the LSU distribution is required; this scheduled operation must inform the component about the rewards to be distributed.
* Each user can withdraw his rewards whenever he wants.

## Known limitations
* Do not use more than 100 different rewards for stakers or the withdraw transactions may fail.
* Do not assign a reward to more than 100 users in a single transaction or it may fail (issue multiple transactions each one containing a single assign\_rewards method call).

## Below are the transaction manifests needed to use this contract:

### Instantiate the component (Mainnet)
```
CALL_FUNCTION
    Address("package_rdx1pk3n6vr996fsws5s8zahjtvytm6sa8vdd5yz9pprjv8h4m0ehm5g2k")
    "AddixFomoRewards"
    "new"
    Address("<OWNER_BADGE>")
    Address("<AIRDROPPER_BADGE>")
;
```

### Mint an user badge
```
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "mint_user_nft"
;
CALL_METHOD
    Address("<ACCOUNT>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```

### Deposit rewards (owner only)
```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<OWNER_BADGE>")
    Decimal("1")
;
CALL_METHOD
    Address("<ACCOUNT>")
    "withdraw"
    Address("<REWARDS_RESOURCE_ADDRESS>")
    Decimal("<AMOUNT_TO_DEPOSIT>")
;
TAKE_ALL_FROM_WORKTOP
    Address("<REWARDS_RESOURCE_ADDRESS>")
    Bucket("rewards")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "deposit_future_rewards"
    Bucket("rewards")
;
```

### Assign rewards to users (an airdropper badge is needed)
```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_amount"
    Address("<AIRDROPPER_BADGE>")
    Decimal("1")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "assign_rewards"
    Map<U64, Decimal>(<USER_BADGE_ID>u64 => Decimal("<AMOUNT>"), <USER_BADGE_ID>u64  => Decimal("<AMOUNT>")...)
    Address("<REWARDS_RESOURCE_ADDRESS>")
;
```

### Withdraw rewards (registered users only)
```
CALL_METHOD
    Address("<ACCOUNT>")
    "create_proof_of_non_fungibles"
    Address("<USER_BADGE>")
    Array<NonFungibleLocalId>(NonFungibleLocalId("#<USER_BADGE_ID>#"))
;
POP_FROM_AUTH_ZONE
    Proof("proof")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "withdraw_rewards"
    Proof("proof")
;
CALL_METHOD
    Address("<ACCOUNT>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
```
