#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::BankrollState;
use bankroll::{
    BankrollMessage, BankrollOperation, BankrollParameters, BankrollResponse,
    DebtRecord, DebtStatus, PublicChainBalances, StakerInfo,
    TokenPotRecord,
};
use linera_sdk::linera_base_types::{Amount, ChainId};
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};

pub struct BankrollContract {
    state: BankrollState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(BankrollContract);

impl WithContractAbi for BankrollContract {
    type Abi = bankroll::BankrollAbi;
}

impl Contract for BankrollContract {
    type Message = BankrollMessage;
    type Parameters = BankrollParameters;
    type InstantiationArgument = ();
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = BankrollState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        BankrollContract { state, runtime }
    }

    async fn instantiate(&mut self, _argument: Self::InstantiationArgument) {
        // validate that the application parameters were configured correctly.
        self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            // * User Chain
            BankrollOperation::Balance { owner } => {
                log::info!("\n\nBankrollOperation::Balance");
                log::info!("BankrollOperation::Balance request from  {:?}", owner);

                let balance_async = self.state.accounts.get(&owner).await;
                let mut balance = balance_async.expect("unable to get balance").unwrap_or_default();

                let daily_bonus = self.state.daily_bonus.get_mut();
                if daily_bonus.is_zero() {
                    daily_bonus.update_bonus(self.runtime.application_parameters().bonus);
                }
                balance.saturating_add_assign(daily_bonus.claim_bonus(self.runtime.system_time()));

                self.state.accounts.insert(&owner, balance).unwrap_or_else(|_| {
                    panic!("unable to update {:?} balance", owner);
                });

                log::info!("BankrollOperation::Balance returning balance: {} for owner: {:?}", balance, owner);
                BankrollResponse::Balance(balance)
            }
            BankrollOperation::UpdateBalance { owner, amount } => {
                log::info!("\n\nBankrollOperation::UpdateBalance");
                log::info!("BankrollOperation::UpdateBalance request from {:?}, updating balance to: {}", owner, amount);

                self.state.accounts.insert(&owner, amount).unwrap_or_else(|_| {
                    panic!("unable to update {:?} balance", owner);
                });

                log::info!("BankrollOperation::UpdateBalance completed for owner: {:?}, new balance: {}", owner, amount);
                BankrollResponse::Ok
            }
            BankrollOperation::NotifyDebt { amount, target_chain, game_type } => {
                log::info!("\n\nBankrollOperation::NotifyDebt");
                log::info!(
                    "BankrollOperation::NotifyDebt request from {:?}, amount: {}, target_chain: {:?}, game_type: {}",
                    self.runtime.authenticated_signer(),
                    amount,
                    target_chain,
                    game_type
                );

                let user_chain = self.runtime.chain_id();
                let created_at = self.runtime.system_time();
                let debt_id = created_at.micros();

                // Create debt record before sending notification
                let debt_record = DebtRecord {
                    id: debt_id,
                    user_chain,
                    amount,
                    created_at,
                    paid_at: None,
                    status: DebtStatus::Pending,
                    game_type: game_type.clone(),
                };

                self.state.debt_log.insert(&debt_id, debt_record.clone()).unwrap_or_else(|_| {
                    panic!("Failed to create debt record for debt_id: {}", debt_id);
                });

                log::info!("Created debt record: {:?}", debt_record);

                self.message_manager(target_chain, BankrollMessage::DebtNotif { debt_id, amount, created_at, game_type });
                log::info!("Sent DebtNotif message to target_chain: {:?}, debt_id: {}", target_chain, debt_id);
                BankrollResponse::Ok
            }
            BankrollOperation::TransferTokenPot { amount, target_chain, game_type } => {
                log::info!("\n\nBankrollOperation::TransferTokenPot");
                log::info!(
                    "BankrollOperation::TransferTokenPot request from {:?}, amount: {}, target_chain: {:?}, game_type: {}",
                    self.runtime.authenticated_signer(),
                    amount,
                    target_chain,
                    game_type
                );

                self.message_manager(target_chain, BankrollMessage::TokenPot { amount, game_type });
                log::info!("Sent TokenPot message to target_chain: {:?}, amount: {}", target_chain, amount);
                BankrollResponse::Ok
            }
            // * Master Chain
            BankrollOperation::MintToken { chain_id, amount } => {
                log::info!("\n\nBankrollOperation::MintToken");
                assert_eq!(
                    self.runtime.chain_id(),
                    self.runtime.application_parameters().master_chain,
                    "MasterChain Authorization Required for BankrollOperation::MintToken"
                );
                log::info!(
                    "BankrollOperation::MintToken request from {:?}, minting {} tokens for chain: {:?}",
                    self.runtime.authenticated_signer(),
                    amount,
                    chain_id
                );
                self.message_manager(chain_id, BankrollMessage::TokenIssued { amount });
                log::info!("Sent TokenIssued message to chain: {:?}, amount: {}", chain_id, amount);

                let data = PublicChainBalances { chain: chain_id, amount };
                self.state.balances.insert(&chain_id, data).unwrap_or_else(|_| {
                    panic!("Failed to update record for Public Chain ID: {}", chain_id);
                });

                BankrollResponse::Ok
            }
            // === Staking Operations ===
            BankrollOperation::Stake { amount } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::Stake from {:?}, amount: {}", owner, amount);
                
                // Check minimum stake amount
                let config = self.state.casino_config.get();
                if amount < config.min_stake_amount {
                    return BankrollResponse::Error(format!("Minimum stake is {}", config.min_stake_amount));
                }
                
                // Get user balance
                let balance = self.state.accounts.get(&owner).await.expect("Failed to get balance").unwrap_or(Amount::ZERO);
                if balance < amount {
                    return BankrollResponse::Error("Insufficient balance".to_string());
                }
                
                // Deduct from balance
                let new_balance = balance.saturating_sub(amount);
                self.state.accounts.insert(&owner, new_balance).expect("Failed to update balance");
                
                // Update staking pool
                let mut pool = self.state.staking_pool.get().clone();
                let current_time = self.runtime.system_time();
                
                // Get or create staker info
                let existing_staker = self.state.stakers.get(&owner).await.expect("Failed to get staker");
                let mut staker_info = match existing_staker {
                    Some(mut info) => {
                        // Calculate and add pending rewards before adding new stake
                        info.unclaimed_rewards = info.earned(pool.reward_per_token);
                        info.staked_amount.saturating_add_assign(amount);
                        info.reward_per_token_paid = pool.reward_per_token;
                        info
                    }
                    None => {
                        pool.staker_count += 1;
                        StakerInfo::new(owner, amount, current_time)
                    }
                };
                staker_info.stake_timestamp = current_time;
                
                pool.total_staked.saturating_add_assign(amount);
                pool.last_update_time = current_time;
                
                self.state.staking_pool.set(pool);
                self.state.stakers.insert(&owner, staker_info.clone()).expect("Failed to update staker");
                
                log::info!("Stake successful. New staked amount: {}", staker_info.staked_amount);
                BankrollResponse::StakingInfo(staker_info)
            }
            BankrollOperation::Unstake { amount } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::Unstake from {:?}, amount: {}", owner, amount);
                
                // Get staker info
                let existing_staker = self.state.stakers.get(&owner).await.expect("Failed to get staker");
                let staker_result = match existing_staker {
                    Some(info) => info,
                    None => return BankrollResponse::Error("No staked amount found".to_string()),
                };
                let mut staker_info = staker_result;
                
                if staker_info.staked_amount < amount {
                    return BankrollResponse::Error("Insufficient staked amount".to_string());
                }
                
                // Update staking pool
                let mut pool = self.state.staking_pool.get().clone();
                
                // Calculate and add pending rewards before unstaking
                staker_info.unclaimed_rewards = staker_info.earned(pool.reward_per_token);
                staker_info.staked_amount = staker_info.staked_amount.saturating_sub(amount);
                staker_info.reward_per_token_paid = pool.reward_per_token;
                
                pool.total_staked = pool.total_staked.saturating_sub(amount);
                pool.last_update_time = self.runtime.system_time();
                
                // Add to user balance
                let balance = self.state.accounts.get(&owner).await.expect("Failed to get balance").unwrap_or(Amount::ZERO);
                let new_balance = balance.saturating_add(amount);
                self.state.accounts.insert(&owner, new_balance).expect("Failed to update balance");
                
                self.state.staking_pool.set(pool);
                self.state.stakers.insert(&owner, staker_info.clone()).expect("Failed to update staker");
                
                log::info!("Unstake successful. Remaining staked: {}", staker_info.staked_amount);
                BankrollResponse::StakingInfo(staker_info)
            }
            BankrollOperation::ClaimStakingRewards => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::ClaimStakingRewards from {:?}", owner);
                
                // Get staker info
                let existing_staker = self.state.stakers.get(&owner).await.expect("Failed to get staker");
                let mut staker_info = match existing_staker {
                    Some(info) => info,
                    None => return BankrollResponse::Error("No staking position found".to_string()),
                };
                
                let pool = self.state.staking_pool.get();
                let rewards = staker_info.earned(pool.reward_per_token);
                
                if rewards == Amount::ZERO {
                    return BankrollResponse::Error("No rewards to claim".to_string());
                }
                
                // Reset rewards tracking
                staker_info.unclaimed_rewards = Amount::ZERO;
                staker_info.reward_per_token_paid = pool.reward_per_token;
                staker_info.last_reward_claim = self.runtime.system_time();
                
                // Add rewards to balance
                let balance = self.state.accounts.get(&owner).await.expect("Failed to get balance").unwrap_or(Amount::ZERO);
                let new_balance = balance.saturating_add(rewards);
                self.state.accounts.insert(&owner, new_balance).expect("Failed to update balance");
                
                self.state.stakers.insert(&owner, staker_info.clone()).expect("Failed to update staker");
                
                log::info!("Claimed rewards: {}", rewards);
                BankrollResponse::StakingInfo(staker_info)
            }
            BankrollOperation::GetStakingInfo { owner } => {
                log::info!("BankrollOperation::GetStakingInfo for {:?}", owner);
                
                let staker = self.state.stakers.get(&owner).await.expect("Failed to get staker");
                match staker {
                    Some(info) => BankrollResponse::StakingInfo(info),
                    None => BankrollResponse::StakingInfo(StakerInfo::default()),
                }
            }
            // === Responsible Gaming Operations ===
            BankrollOperation::SetDailyLossLimit { limit } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::SetDailyLossLimit for {:?}, limit: {}", owner, limit);
                
                let existing = self.state.player_limits.get(&owner).await.expect("Failed to get limits");
                let mut limits = existing.unwrap_or_default();
                limits.daily_loss_limit = Some(limit);
                
                self.state.player_limits.insert(&owner, limits.clone()).expect("Failed to update limits");
                BankrollResponse::PlayerLimits(limits)
            }
            BankrollOperation::SetMaxSingleBet { limit } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::SetMaxSingleBet for {:?}, limit: {}", owner, limit);
                
                let existing = self.state.player_limits.get(&owner).await.expect("Failed to get limits");
                let mut limits = existing.unwrap_or_default();
                limits.max_single_bet = Some(limit);
                
                self.state.player_limits.insert(&owner, limits.clone()).expect("Failed to update limits");
                BankrollResponse::PlayerLimits(limits)
            }
            BankrollOperation::SelfExclude { duration_days } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::SelfExclude for {:?}, days: {}", owner, duration_days);
                
                let existing = self.state.player_limits.get(&owner).await.expect("Failed to get limits");
                let mut limits = existing.unwrap_or_default();
                limits.set_self_exclusion(duration_days, self.runtime.system_time());
                
                self.state.player_limits.insert(&owner, limits.clone()).expect("Failed to update limits");
                BankrollResponse::PlayerLimits(limits)
            }
            BankrollOperation::RemoveSelfExclusion => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::RemoveSelfExclusion for {:?}", owner);
                
                let existing = self.state.player_limits.get(&owner).await.expect("Failed to get limits");
                let mut limits = match existing {
                    Some(l) => l,
                    None => return BankrollResponse::Error("No limits found".to_string()),
                };
                
                match limits.remove_self_exclusion(self.runtime.system_time()) {
                    Ok(()) => {
                        self.state.player_limits.insert(&owner, limits.clone()).expect("Failed to update limits");
                        BankrollResponse::PlayerLimits(limits)
                    }
                    Err(e) => BankrollResponse::Error(e),
                }
            }
            BankrollOperation::GetPlayerLimits { owner } => {
                log::info!("BankrollOperation::GetPlayerLimits for {:?}", owner);
                
                let limits = self.state.player_limits.get(&owner).await.expect("Failed to get limits").unwrap_or_default();
                BankrollResponse::PlayerLimits(limits)
            }
            // === Admin Operations ===
            BankrollOperation::SetGlobalConfig { config } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::SetGlobalConfig from {:?}", owner);
                
                // Check if caller is admin
                let access = self.state.access_control.get();
                if !access.is_admin(&owner) {
                    return BankrollResponse::Error("Admin access required".to_string());
                }
                
                self.state.casino_config.set(config);
                log::info!("Global config updated");
                BankrollResponse::Ok
            }
            BankrollOperation::PauseGame { game_type } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::PauseGame from {:?}, game: {}", owner, game_type);
                
                let access = self.state.access_control.get();
                if !access.is_operator(&owner) {
                    return BankrollResponse::Error("Operator access required".to_string());
                }
                
                let mut config = self.state.casino_config.get().clone();
                if !config.paused_games.contains(&game_type) {
                    config.paused_games.push(game_type.clone());
                }
                self.state.casino_config.set(config);
                
                log::info!("Game {} paused", game_type);
                BankrollResponse::Ok
            }
            BankrollOperation::UnpauseGame { game_type } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::UnpauseGame from {:?}, game: {}", owner, game_type);
                
                let access = self.state.access_control.get();
                if !access.is_operator(&owner) {
                    return BankrollResponse::Error("Operator access required".to_string());
                }
                
                let mut config = self.state.casino_config.get().clone();
                config.paused_games.retain(|g| g != &game_type);
                self.state.casino_config.set(config);
                
                log::info!("Game {} unpaused", game_type);
                BankrollResponse::Ok
            }
            BankrollOperation::SetAdmin { new_admin } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::SetAdmin from {:?}, new_admin: {:?}", owner, new_admin);
                
                let mut access = self.state.access_control.get().clone();
                
                // Only existing admin can set new admin (or if no admin exists yet)
                if access.admin.is_some() && !access.is_admin(&owner) {
                    return BankrollResponse::Error("Only current admin can set new admin".to_string());
                }
                
                access.admin = Some(new_admin);
                self.state.access_control.set(access);
                
                log::info!("Admin updated");
                BankrollResponse::Ok
            }
            BankrollOperation::AddOperator { operator } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::AddOperator from {:?}, operator: {:?}", owner, operator);
                
                let mut access = self.state.access_control.get().clone();
                if !access.is_admin(&owner) {
                    return BankrollResponse::Error("Admin access required".to_string());
                }
                
                access.add_operator(operator);
                self.state.access_control.set(access);
                
                log::info!("Operator added");
                BankrollResponse::Ok
            }
            BankrollOperation::RemoveOperator { operator } => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::RemoveOperator from {:?}, operator: {:?}", owner, operator);
                
                let mut access = self.state.access_control.get().clone();
                if !access.is_admin(&owner) {
                    return BankrollResponse::Error("Admin access required".to_string());
                }
                
                access.remove_operator(&operator);
                self.state.access_control.set(access);
                
                log::info!("Operator removed");
                BankrollResponse::Ok
            }
            BankrollOperation::DistributeStakingRewards => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                log::info!("BankrollOperation::DistributeStakingRewards from {:?}", owner);
                
                let access = self.state.access_control.get();
                if !access.is_operator(&owner) {
                    return BankrollResponse::Error("Operator access required".to_string());
                }
                
                let mut pool = self.state.staking_pool.get().clone();
                if pool.total_profit == Amount::ZERO {
                    return BankrollResponse::Error("No profits to distribute".to_string());
                }
                
                // The profit has already been added to reward_per_token via add_profit()
                // Reset total_profit for next distribution cycle
                pool.total_profit = Amount::ZERO;
                pool.last_update_time = self.runtime.system_time();
                self.state.staking_pool.set(pool);
                
                log::info!("Staking rewards distributed");
                BankrollResponse::Ok
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        let origin_chain_id = self.runtime.message_origin_chain_id().expect("Chain ID missing from message");

        match message {
            // * Public Chain
            BankrollMessage::TokenIssued { amount } => {
                log::info!("\n\nBankrollMessage::TokenIssued");
                log::info!(
                    "BankrollMessage::TokenIssued from {:?} at {:?}, amount: {}",
                    origin_chain_id,
                    self.runtime.chain_id(),
                    amount
                );
                let current_token = self.state.casino_token.get_mut();
                let previous_balance = *current_token;
                current_token.saturating_add_assign(amount);
                log::info!("Token balance updated: {} -> {}", previous_balance, current_token);
            }
            BankrollMessage::DebtNotif { debt_id, amount, created_at, game_type } => {
                log::info!("\n\nBankrollMessage::DebtNotif");
                log::info!(
                    "BankrollMessage::DebtNotif debt_id: {} from user_chain: {:?} amount: {} game_type: {} at {:?}",
                    debt_id,
                    origin_chain_id,
                    amount,
                    game_type,
                    self.runtime.chain_id()
                );

                // Verify we have sufficient tokens
                let current_token = self.state.casino_token.get();
                log::info!("Current token pool before debt payment: {}", current_token);
                assert!(
                    *current_token >= amount,
                    "Insufficient tokens to pay debt. Available: {}, Required: {}",
                    current_token,
                    amount
                );

                // Subtract the debt amount from casino_token pool
                let current_token_log = current_token.clone();
                let remaining_token = current_token.saturating_sub(amount);
                self.state.casino_token.set(remaining_token);

                log::info!(
                    "Debt payment processed. Token pool: {} -> {}. Sending DebtPaid to {:?}",
                    current_token_log,
                    remaining_token,
                    origin_chain_id
                );

                // Send DebtPaid message back to the user chain
                let paid_at = self.runtime.system_time();
                self.message_manager(origin_chain_id, BankrollMessage::DebtPaid { debt_id, amount, paid_at });

                // Log debt history
                let debt_record = DebtRecord {
                    id: debt_id,
                    user_chain: origin_chain_id,
                    amount,
                    created_at,
                    paid_at: Some(paid_at),
                    status: DebtStatus::Paid,
                    game_type,
                };
                self.state.debt_log.insert(&debt_id, debt_record.clone()).unwrap_or_else(|_| {
                    panic!("Failed to create debt record for debt_id: {}", debt_id);
                });
            }
            BankrollMessage::TokenPot { amount, game_type } => {
                log::info!("\n\nBankrollMessage::TokenPot");
                log::info!(
                    "BankrollMessage::TokenPot from {:?} amount: {} game_type: {} at {:?}",
                    origin_chain_id,
                    amount,
                    game_type,
                    self.runtime.chain_id()
                );

                // Add the pot amount to casino_token pool
                let current_token = self.state.casino_token.get_mut();
                current_token.saturating_add_assign(amount);

                // Create token pot record for history
                let created_at = self.runtime.system_time();
                let pot_id = created_at.micros();
                let pot_record = TokenPotRecord {
                    id: pot_id,
                    user_chain: origin_chain_id,
                    amount,
                    created_at,
                    game_type,
                };

                self.state.token_pot_log.insert(&pot_id, pot_record.clone()).unwrap_or_else(|_| {
                    panic!("Failed to create token pot record for pot_id: {}", pot_id);
                });

                log::info!("Token pot received. New total tokens: {}. Pot record created: {:?}", current_token, pot_record);
            }
            // * User Chain
            BankrollMessage::DebtPaid { debt_id, amount, paid_at } => {
                log::info!("\n\nBankrollMessage::DebtPaid");
                log::info!(
                    "BankrollMessage::DebtPaid debt_id: {} amount: {} timestamp: {:?} at {:?}",
                    debt_id,
                    amount,
                    paid_at,
                    self.runtime.chain_id()
                );

                // Update the debt record with paid_at and status
                let mut debt_record = self
                    .state
                    .debt_log
                    .get(&debt_id)
                    .await
                    .expect("Failed to get debt record")
                    .expect("Debt record not found");

                debt_record.paid_at = Some(paid_at);
                debt_record.status = DebtStatus::Paid;

                self.state.debt_log.insert(&debt_id, debt_record).unwrap_or_else(|_| {
                    panic!("Failed to update debt record for debt_id: {}", debt_id);
                });

                log::info!("Debt {} successfully updated to Paid status", debt_id);
            }
            // * Master Chain
            BankrollMessage::TokenUpdate { amount } => {
                log::info!("\n\nBankrollMessage::TokenUpdate");
                log::info!(
                    "BankrollMessage::TokenUpdate from {:?} amount: {} at {:?}",
                    origin_chain_id,
                    amount,
                    self.runtime.chain_id()
                );

                let data = PublicChainBalances {
                    chain: origin_chain_id,
                    amount,
                };
                self.state.balances.insert(&origin_chain_id, data).unwrap_or_else(|_| {
                    panic!("Failed to update record for Public Chain ID: {}", origin_chain_id);
                });
            }
            // === Staking Messages ===
            BankrollMessage::StakingRewardDistribution { amount_per_token } => {
                log::info!(
                    "BankrollMessage::StakingRewardDistribution from {:?} amount_per_token: {}",
                    origin_chain_id,
                    amount_per_token
                );
                // TODO: Update staker rewards based on amount_per_token
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl BankrollContract {
    fn message_manager(&mut self, destination: ChainId, message: BankrollMessage) {
        self.runtime.prepare_message(message).with_tracking().send_to(destination);
    }
}

