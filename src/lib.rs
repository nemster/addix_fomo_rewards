use scrypto::prelude::*;

static USER_NFT_NAME: &str = "TODO";
static USER_NFT_ICON: &str = "https://TODO";

#[derive(Debug, ScryptoSbor, NonFungibleData)]
struct UserNftData {
    creation_date: Instant,
    #[mutable]
    last_rewards_withdraw: Instant,
}

#[derive(ScryptoSbor, ScryptoEvent)]
struct NewUserNftEvent {
    id: u64,
}

#[blueprint]
#[types(u64, Decimal, UserNftData)]
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
        rewards_vault: Vault,
        assigned_rewards_quantity: Decimal,
        assigned_rewards: KeyValueStore<u64, Decimal>,
    }

    impl AddixFomoRewards {

        pub fn new(
                owner_badge_address: ResourceAddress,
                rewards_address: ResourceAddress,
                airdropper_badge_address: ResourceAddress,
            ) -> Global<AddixFomoRewards> {

            // Reserve a ComponentAddress for setting rules on resources
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(AddixFomoRewards::blueprint_id());

            let user_nft_resource_manager = ResourceBuilder::new_integer_non_fungible_with_registered_type::<UserNftData>(
                OwnerRole::Updatable(rule!(require(owner_badge_address)))
            )
            .metadata(metadata!(
                roles {
                    metadata_setter => rule!(require(owner_badge_address));
                    metadata_setter_updater => rule!(require(owner_badge_address));
                    metadata_locker => rule!(require(owner_badge_address));
                    metadata_locker_updater => rule!(require(owner_badge_address));
                },
                init {
                    "name" => USER_NFT_NAME.to_string(), updatable;
                    "icon_url" => MetadataValue::Url(UncheckedUrl::of(USER_NFT_ICON.to_string())), updatable;
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

            Self {
                user_nft_resource_manager: user_nft_resource_manager,
                last_user_nft_id: 0,
                rewards_vault: Vault::new(rewards_address),
                assigned_rewards_quantity: Decimal::ZERO,
                assigned_rewards: KeyValueStore::new_with_registered_type(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .roles(roles!(
                airdropper => rule!(require(airdropper_badge_address));
            ))
            .with_address(address_reservation)
            .globalize()
        }

        pub fn mint_user_nft(&mut self) -> Bucket {
            self.last_user_nft_id += 1;

            self.assigned_rewards.insert(
                self.last_user_nft_id,
                Decimal::ZERO
            );

            let now = Clock::current_time_rounded_to_seconds();

            Runtime::emit_event(NewUserNftEvent {
                id: self.last_user_nft_id,
            });

            self.user_nft_resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(self.last_user_nft_id.into()),
                UserNftData {
                    creation_date: now,
                    last_rewards_withdraw: now,
                }
            )
        }

        pub fn deposit_future_rewards(
            &mut self,
            future_rewards: Bucket
        ) {
            self.rewards_vault.put(future_rewards);
        }

        pub fn assign_rewards(
            &mut self,
            users: Vec<u64>,
            rewards: Vec<Decimal>,
        ) {
            assert!(
                users.len() == rewards.len(),
                "user and rewards have different lenght"
            );

            for i in 0 .. users.len() {
                let user = users[i];
                assert!(
                    user > 0 && user <= self.last_user_nft_id,
                    "User out of bounds: {}",
                    user
                );

                let reward = rewards[i];
                assert!(
                    reward > Decimal::ZERO,
                    "Reward below or equal to zero: {}",
                    reward
                );

                *self.assigned_rewards.get_mut(&user).unwrap() += reward;

                self.assigned_rewards_quantity += reward;
            }

            assert!(self.rewards_vault.amount() >= self.assigned_rewards_quantity,
                "assigned rewards > available rewards"
            );
        }

        pub fn withdraw_rewards(
            &mut self,
            user_proof: Proof
        ) -> Bucket {
            let checked_proof = user_proof.check_with_message(
                self.user_nft_resource_manager.address(),
                "Incorrect proof",
            ).as_non_fungible();

            let user_nft_data = checked_proof.non_fungible::<UserNftData>().data();

            // TODO: withdraw rewards, mark rewards, update data
        }
    }
}
