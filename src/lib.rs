use scrypto::prelude::*;

// Struct representing one of the coins to be used as rewards
#[derive(ScryptoSbor)]
struct Reward {
    vault: Vault,
    total_assigned: Decimal,
    assigned: KeyValueStore<u64, Decimal>,
}

// NonFungibleData of an user badge
#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct UserNftData {
    id: u64,
    creation_date: Instant,
    #[mutable]
    last_rewards_withdraw: Instant,
}

// Event emitted at each user badge creation
#[derive(ScryptoSbor, ScryptoEvent)]
struct NewUserNftEvent {
    id: u64,
}

#[blueprint]
#[types(u64, Decimal, UserNftData, Reward)]
#[events(NewUserNftEvent)]
mod addix_fomo_rewards {

    enable_method_auth! {
        roles {
            airdropper => updatable_by: [OWNER];
        },
        methods {
            mint_user_nft => PUBLIC;
            deposit_future_rewards => restrict_to: [OWNER];
            assign_rewards => restrict_to: [airdropper];
            withdraw_rewards => PUBLIC;
        }
    }

    struct AddixFomoRewards {
        user_nft_resource_manager: ResourceManager,
        last_user_nft_id: u64,
        rewards: Vec<Reward>,
    }

    impl AddixFomoRewards {

        // Instantiate a new AddixFomoRewards component and globalize it
        pub fn new(
                owner_badge_address: ResourceAddress,
                airdropper_badge_address: ResourceAddress,
            ) -> Global<AddixFomoRewards> {

            // Reserve a ComponentAddress for setting rules on resources
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(AddixFomoRewards::blueprint_id());

            // Create a ResourceManager to mint user badges
            let user_nft_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<UserNftData>(
                OwnerRole::Updatable(rule!(require(owner_badge_address)))
            )
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                }
            ))
            .mint_roles(mint_roles!(
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(require(owner_badge_address));
            ))
            .non_fungible_data_update_roles(non_fungible_data_update_roles!(
                non_fungible_data_updater => rule!(require(global_caller(component_address)));
                non_fungible_data_updater_updater => rule!(require(owner_badge_address));
            ))
            .burn_roles(burn_roles!(
                burner => rule!(require(global_caller(component_address)));
                burner_updater => rule!(require(owner_badge_address));
            ))
            .withdraw_roles(withdraw_roles!(
                withdrawer => rule!(deny_all); // Non transferable
                withdrawer_updater => rule!(require(owner_badge_address));
            ))
            .recall_roles(recall_roles!(
                recaller => rule!(require(global_caller(component_address))); // Recallable
                recaller_updater => rule!(require(owner_badge_address));
            ))
            .create_with_no_initial_supply();

            // Instantiate component
            Self {
                user_nft_resource_manager: user_nft_resource_manager,
                last_user_nft_id: 0,
                rewards: vec![],
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                airdropper => rule!(require(airdropper_badge_address));
            ))
            .with_address(address_reservation)
            .globalize()
        }

        // Mint a new user badge
        pub fn mint_user_nft(&mut self) -> Bucket {

            // Increase the unique id
            self.last_user_nft_id += 1;

            // Get the current time
            let now = Clock::current_time_rounded_to_seconds();

            // Emit the NewUserNftEvent event to help bot development
            Runtime::emit_event(NewUserNftEvent {
                id: self.last_user_nft_id,
            });

            // Mint the NFT and return it
            self.user_nft_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_user_nft_id.into()),
                UserNftData {
                    id: self.last_user_nft_id,
                    creation_date: now,
                    last_rewards_withdraw: now,
                }
            )
        }

        // The OWNER can call this method to deposit future staking rewards for stakers
        // The blueprint supports multiple different coins at the same time (make sure to not
        // exceed 100 different coins or the transactions may fail)
        pub fn deposit_future_rewards(
            &mut self,
            future_rewards: Bucket
        ) {
            // If there's already a Reward struct for this coin, just use it
            for i in 0 .. self.rewards.len() {
                if self.rewards[i].vault.resource_address() == future_rewards.resource_address() {
                    self.rewards[i].vault.put(future_rewards);
                    return;
                }
            }

            // If not, add a new Reward struct to the list
            self.rewards.push(
                Reward {
                    vault: Vault::with_bucket(future_rewards),
                    total_assigned: Decimal::ZERO,
                    assigned: KeyValueStore::new_with_registered_type(),
                }
            );
        }

        // The airdropper bot can use this method to assing the previously deposited rewards to
        // registered users.
        // In case there are more than 100 registered stakers, you better call this method multiple
        // times.
        // The method checks that the total number of assigned coins for each resource do not
        // exceed the deposited number of coins, you have to call the deposit_future_rewards method
        // before this one.
        pub fn assign_rewards(
            &mut self,
            users: Vec<u64>,
            amounts: Vec<Decimal>,
            coin: ResourceAddress,
        ) {
            assert!(
                users.len() == amounts.len(),
                "users and amounts have different lenght"
            );

            // Search the coin in the list
            for j in 0 .. self.rewards.len() {
                if coin == self.rewards[j].vault.resource_address() {

                    // For each user id in the Vec
                    for i in 0 .. users.len() {
                        let user = users[i];

                        // Check that the user exists
                        assert!(
                            user > 0 && user <= self.last_user_nft_id,
                            "User out of bounds: {}",
                            user
                        );

                        let reward = amounts[i];
                        assert!(
                            reward > Decimal::ZERO,
                            "Reward below or equal to zero: {}",
                            reward
                        );

                        // Assign the reward to the user, eventually adding a new item to the
                        // assigned KeyValueStore
                        if self.rewards[j].assigned.get(&user).is_some() {
                            *self.rewards[j].assigned.get_mut(&user).unwrap() += reward;
                        } else {
                            self.rewards[j].assigned.insert(user, reward);
                        }

                        // Update the total number of assigned coins
                        self.rewards[j].total_assigned += reward;
                    }

                    // Make sure the total of assigned coins is not greater than the total
                    // deposited coins
                    assert!(self.rewards[j].vault.amount() >= self.rewards[j].total_assigned,
                        "assigned rewards > available rewards"
                    );

                    return;
                }
            }

            // This coin has never beed deposited
            Runtime::panic("Coin not found".to_string());
        }

        // Anyone with a user badge can use this method to withdraw his rewards.
        pub fn withdraw_rewards(
            &mut self,
            user_proof: Proof
        ) -> Vec<Bucket> {

            // Check that the proof is valid
            let checked_proof = user_proof.check_with_message(
                self.user_nft_resource_manager.address(),
                "Incorrect proof",
            ).as_non_fungible();

            // Get the NonFungibleData for the user badge
            let user_nft_data = checked_proof.non_fungible::<UserNftData>().data();

            // Update the last_rewards_withdraw in the user badge
            self.user_nft_resource_manager.update_non_fungible_data(
                &NonFungibleLocalId::Integer(user_nft_data.id.into()),
                "last_rewards_withdraw",
                Clock::current_time_rounded_to_seconds(),
            );

            // Create a Vec of buckets to return to the user
            let mut buckets: Vec<Bucket> = vec![];

            // For each coin
            for i in 0 .. self.rewards.len() {
                let reward = &mut self.rewards[i];

                // If the user has beed assigned this coin as reward
                if reward.assigned.get(&user_nft_data.id).is_some() {

                    // Take the rewards from the vault
                    let bucket = reward.vault.take_advanced(
                        *reward.assigned.get(&user_nft_data.id).unwrap(),
                        WithdrawStrategy::Rounded(RoundingMode::ToZero)
                    );

                    // Update the user assigned rewards amount
                    *reward.assigned.get_mut(&user_nft_data.id).unwrap() -= bucket.amount();

                    // Update the total assigned rewards number
                    reward.total_assigned -= bucket.amount();

                    // Add the bucket to the list
                    buckets.push(bucket);
                }
            }

            // Return all of the reward buckets to the user
            buckets
        }
    }
}
