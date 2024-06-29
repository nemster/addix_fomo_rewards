use scrypto_test::prelude::*;
use addix_fomo_rewards::addix_fomo_rewards_test::*;

#[test]
fn test_addix_fomo_rewards() -> Result<(), RuntimeError> {
    let mut env = TestEnvironment::new();
    env.disable_auth_module();
    let package_address = PackageFactory::compile_and_publish(this_package!(), &mut env, CompileProfile::Fast)?;

    // Create the owner badge
    let owner_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)?;
    let owner_badge_address = owner_badge_bucket.resource_address(&mut env)?;

    // Create the airdropper badge
    let airdropper_badge_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(0)
        .mint_initial_supply(1, &mut env)?;
    let airdropper_badge_address = airdropper_badge_bucket.resource_address(&mut env)?;

    // Instantiate a AddixFomoRewards component
    let mut addix_fomo_rewards = AddixFomoRewards::new(
        owner_badge_address,
        airdropper_badge_address,
        package_address,
        &mut env
    )?;

    // Create the reward1 coins
    let reward1_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1000000, &mut env)?;
    let reward1_address = reward1_bucket.resource_address(&mut env)?;

    // Deposit 6 reward1 coins
    addix_fomo_rewards.deposit_future_rewards(
        reward1_bucket.take(dec!("6"), &mut env)?,
        &mut env
    )?;

    // Create the reward2 coins
    let reward2_bucket = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(2)
        .mint_initial_supply(1000000, &mut env)?;
    let reward2_address = reward2_bucket.resource_address(&mut env)?;

    // Deposit all of the reward2 coins
    addix_fomo_rewards.deposit_future_rewards(
        reward2_bucket,
        &mut env
    )?;

    // Deposit 7 more reward1 coins
    addix_fomo_rewards.deposit_future_rewards(
        reward1_bucket.take(dec!("7"), &mut env)?,
        &mut env
    )?;

    // Create user NFT #1
    let user_nft_bucket1 = addix_fomo_rewards.mint_user_nft(
        &mut env
    )?;

    // Create user NFT #2
    let user_nft_bucket2 = addix_fomo_rewards.mint_user_nft(
        &mut env
    )?;

    // Distribute reward1 coins to both users
    addix_fomo_rewards.assign_rewards(
        HashMap::from([
            (1, dec!("10")),
            (2, dec!("1"))
        ]),
        reward1_address,
        &mut env
    )?;

    // Distribute reward2 coins to both users
    addix_fomo_rewards.assign_rewards(
        HashMap::from([
            (1, dec!("1.006")),
            (2, dec!("2.006"))
        ]),
        reward2_address,
        &mut env
    )?;

    // Distribute more reward2 coins to both users
    addix_fomo_rewards.assign_rewards(
        HashMap::from([
            (1, dec!("1")),
            (2, dec!("1.005"))
        ]),
        reward2_address,
        &mut env
    )?;

    // Get rewards for user1
    let ids1 = user_nft_bucket1.non_fungible_local_ids(&mut env)?;
    let user1_proof = user_nft_bucket1.create_proof_of_non_fungibles(ids1, &mut env)?;
    let user1_rewards = addix_fomo_rewards.withdraw_rewards(user1_proof, &mut env)?;
    let user1_reward1_address = user1_rewards[0].resource_address(&mut env)?;
    let user1_reward2_address = user1_rewards[1].resource_address(&mut env)?;
    let user1_reward1_amount = user1_rewards[0].amount(&mut env)?;
    let user1_reward2_amount = user1_rewards[1].amount(&mut env)?;
    assert!(
        user1_reward1_address == reward1_address,
        "wrong reward1 received by user #1"
    );
    assert!(
        user1_reward2_address == reward2_address,
        "wrong reward2 received by user #1"
    );
    assert!(
        user1_reward1_amount == dec!("10"),
        "wrong reward1 amount received by user #1: {}",
        user1_reward1_amount
    );
    assert!(
        user1_reward2_amount == dec!("2"), // 1.006 + 1 truncated to 2 decimal digits
        "wrong reward2 amount received by user #1: {}",
        user1_reward2_amount
    );

    // Get rewards for user2
    let ids2 = user_nft_bucket2.non_fungible_local_ids(&mut env)?;
    let user2_proof = user_nft_bucket2.create_proof_of_non_fungibles(ids2, &mut env)?;
    let user2_rewards = addix_fomo_rewards.withdraw_rewards(user2_proof, &mut env)?;
    let user2_reward1_address = user2_rewards[0].resource_address(&mut env)?;
    let user2_reward2_address = user2_rewards[1].resource_address(&mut env)?;
    let user2_reward1_amount = user2_rewards[0].amount(&mut env)?;
    let user2_reward2_amount = user2_rewards[1].amount(&mut env)?;
    assert!(
        user2_reward1_address == reward1_address,
        "wrong reward1 received by user #2"
    );
    assert!(
        user2_reward2_address == reward2_address,
        "wrong reward2 received by user #2"
    );
    assert!(
        user2_reward1_amount == dec!("1"),
        "wrong reward1 amount received by user #2: {}",
        user2_reward1_amount
    );
    assert!(
        user2_reward2_amount == dec!("3.01"), // 2.006 + 1.005 truncated to 2 decimal digits
        "wrong reward2 amount received by user #2: {}",
        user2_reward2_amount
    );

    // Distribute more reward1 coins to user #2
    addix_fomo_rewards.assign_rewards(
        HashMap::from([(2, dec!("1"))]),
        reward1_address,
        &mut env
    )?;

    // Distribute more reward2 coins to user #2
    addix_fomo_rewards.assign_rewards(
        HashMap::from([(2, dec!("4.0095"))]),
        reward2_address,
        &mut env
    )?;

    // Get more rewards for user2
    let ids2 = user_nft_bucket2.non_fungible_local_ids(&mut env)?;
    let user2_proof = user_nft_bucket2.create_proof_of_non_fungibles(ids2, &mut env)?;
    let user2_reward1_address = user2_rewards[0].resource_address(&mut env)?;
    let user2_reward2_address = user2_rewards[1].resource_address(&mut env)?;
    let user2_rewards = addix_fomo_rewards.withdraw_rewards(user2_proof, &mut env)?;
    let user2_reward1_amount = user2_rewards[0].amount(&mut env)?;
    let user2_reward2_amount = user2_rewards[1].amount(&mut env)?;
    assert!(
        user2_reward1_address == reward1_address,
        "wrong reward1 received by user #2"
    );
    assert!(
        user2_reward2_address == reward2_address,
        "wrong reward2 received by user #2"
    );
    assert!(
        user2_reward1_amount == dec!("1"),
        "wrong reward1 amount received by user #2 in the second round: {}",
        user2_reward1_amount
    );
    assert!(
        user2_reward2_amount == dec!("4.01"), // 2.006 + 1.005 - 3.01 + 4.0095 truncated to 2 decimal digits
        "wrong reward2 amount received by user #2 in the second round: {}",
        user2_reward2_amount
    );

    Ok(())
}
