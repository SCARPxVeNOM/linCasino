#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::RouletteState;
use abi::roulette::{Bet, RouletteGame, UserStatus};
use abi::provably_fair::ProvablyFairRNG;
use bankroll::{BankrollOperation, BankrollResponse};
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};
use roulette::{RouletteEvent, RouletteMessage, RouletteOperation, RouletteParameters};

pub struct RouletteContract {
    state: RouletteState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(RouletteContract);

impl WithContractAbi for RouletteContract {
    type Abi = roulette::RouletteAbi;
}

impl Contract for RouletteContract {
    type Message = RouletteMessage;
    type Parameters = RouletteParameters;
    type InstantiationArgument = u64;
    type EventValue = RouletteEvent;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = RouletteState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        RouletteContract { state, runtime }
    }

    async fn instantiate(&mut self, argument: Self::InstantiationArgument) {
        self.state.instantiate_value.set(argument);
        self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            RouletteOperation::GetBalance {} => {
                let owner = self.runtime.authenticated_signer().expect("No authenticated signer");
                let bankroll_app_id = self.runtime.application_parameters().bankroll;
                let balance_response = self
                    .runtime
                    .call_application(
                        true,
                        bankroll_app_id,
                        &BankrollOperation::Balance { owner },
                    );
                match balance_response {
                    BankrollResponse::Balance(amount) => {
                        let profile = self.state.profile.get_mut();
                        let mut profile_clone = profile.clone();
                        profile_clone.update_balance(amount);
                        profile_clone.calculate_bet_data();
                        let _ = profile;
                        self.state.profile.set(profile_clone);
                    }
                    _ => {}
                }
            }
            RouletteOperation::StartSinglePlayerGame { name: _ } => {
                let mut game = RouletteGame::new();
                let timestamp = self.runtime.system_time();
                // Default 60 second betting period
                game.start_betting(timestamp.micros(), 60_000_000);
                self.state.single_player_game.set(game);
                self.state.user_status.set(UserStatus::InSinglePlayerGame);
            }
            RouletteOperation::PlaceBet { bet_type, amount } => {
                let mut game = self.state.single_player_game.get().clone();
                // Convert Amount to u64 (Amount is in smallest unit, 1 token = 1e9 units)
                // First convert to u128, then to u64
                let amount_u128: u128 = amount.saturating_div(1u128).into();
                let amount_u64: u64 = amount_u128.min(u64::MAX as u128) as u64;
                let current_time_micros = self.runtime.system_time().micros();
                let bet = Bet {
                    bet_type: bet_type.clone(),
                    amount: amount_u64,
                    player_id: 0,
                    placed_at_micros: current_time_micros,
                };
                let _ = game.place_bet(bet);
                self.state.single_player_game.set(game);
            }
            RouletteOperation::Spin {} => {
                let mut game = self.state.single_player_game.get().clone();
                let timestamp = self.runtime.system_time();
                let chain_id_str = format!("{:?}", self.runtime.chain_id());
                
                // Create provably fair RNG for spin
                let server_seed = format!("{}-{}-roulette-{}", chain_id_str, timestamp.micros(), game.round_number);
                let client_seed = game.client_seed.clone().unwrap_or_else(|| format!("round-{}", game.round_number));
                
                let (mut rng, original_seed) = ProvablyFairRNG::new(&server_seed);
                rng.set_client_seed(client_seed);
                
                // Reveal and generate result (0-36)
                let _ = rng.reveal(original_seed.clone());
                let result = rng.generate_result(&original_seed, 0, 37).unwrap_or(0) as u8;
                
                log::info!("Provably fair roulette spin: result={}, server_seed_hash={:?}", result, rng.server_seed_hash);
                
                let _winnings = game.spin(result).unwrap_or_default();
                self.state.single_player_game.set(game);
            }
            RouletteOperation::SetClientSeed { seed } => {
                let mut game = self.state.single_player_game.get().clone();
                game.set_client_seed(seed);
                self.state.single_player_game.set(game);
            }
            RouletteOperation::CommitSeed {} => {
                // Phase 1: Generate seed, store securely, publish ONLY the hash
                let mut game = self.state.single_player_game.get().clone();
                let timestamp = self.runtime.system_time();
                let chain_id_str = format!("{:?}", self.runtime.chain_id());
                
                // Generate server seed
                let server_seed = format!("{}-{}-roulette-{}", chain_id_str, timestamp.micros(), game.round_number + 1);
                let hash = ProvablyFairRNG::hash_seed(&server_seed);
                
                // Store seed securely (NOT exposed via GraphQL)
                self.state.pending_server_seed.set(server_seed);
                
                // Publish only the hash and start betting period
                game.server_seed_hash = Some(hash);
                game.phase = abi::roulette::RoulettePhase::Committed;
                game.start_betting(timestamp.micros(), 60_000_000); // 60 second betting window
                
                log::info!("Commit phase: seed hash published, betting open for 60 seconds");
                self.state.single_player_game.set(game);
            }
            RouletteOperation::RevealAndSpin {} => {
                // Phase 2: After betting closes, reveal seed and generate result
                let mut game = self.state.single_player_game.get().clone();
                let now = self.runtime.system_time().micros();
                
                // Verify betting period has ended
                if now < game.betting_deadline_micros {
                    log::warn!("Cannot reveal while betting is still open - {} micros remaining", 
                        game.betting_deadline_micros - now);
                    return;
                }
                
                // Verify we're in the committed phase
                if game.phase != abi::roulette::RoulettePhase::Committed {
                    log::warn!("Invalid phase for reveal: {:?}", game.phase);
                    return;
                }
                
                // Get the stored seed
                let server_seed = self.state.pending_server_seed.get().clone();
                if server_seed.is_empty() {
                    log::warn!("No pending server seed found");
                    return;
                }
                
                let client_seed = game.client_seed.clone().unwrap_or_else(|| format!("round-{}", game.round_number));
                
                // Verify the seed matches the committed hash
                let computed_hash = ProvablyFairRNG::hash_seed(&server_seed);
                if game.server_seed_hash.as_ref() != Some(&computed_hash) {
                    log::error!("Server seed does not match committed hash!");
                    return;
                }
                
                // Create RNG and generate result
                let (mut rng, _) = ProvablyFairRNG::new(&server_seed);
                rng.set_client_seed(client_seed);
                let result = rng.generate_result(&server_seed, 0, 37).unwrap_or(0) as u8;
                
                // Store the revealed seed for verification
                game.revealed_server_seed = Some(server_seed.clone());
                game.phase = abi::roulette::RoulettePhase::Revealed;
                
                log::info!("Reveal phase: seed={}, result={}", server_seed, result);
                
                // Spin the wheel with the verified random result
                let _winnings = game.spin(result).unwrap_or_default();
                
                // Clear pending seed
                self.state.pending_server_seed.set(String::new());
                self.state.single_player_game.set(game);
            }
            _ => {
                log::info!("Roulette operation not yet implemented: {:?}", operation);
            }
        }
    }

    async fn execute_message(&mut self, _message: Self::Message) {
        // Handle cross-chain messages
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}
