#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::RouletteState;
use abi::roulette::{Bet, RouletteGame, UserStatus};
use abi::random::get_random_value;
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
                let game = RouletteGame::new();
                self.state.single_player_game.set(game);
                self.state.user_status.set(UserStatus::InSinglePlayerGame);
            }
            RouletteOperation::PlaceBet { bet_type, amount } => {
                let mut game = self.state.single_player_game.get().clone();
                // Convert Amount to u64 (Amount is in smallest unit, 1 token = 1e9 units)
                // First convert to u128, then to u64
                let amount_u128: u128 = amount.saturating_div(1u128).into();
                let amount_u64: u64 = amount_u128.min(u64::MAX as u128) as u64;
                let bet = Bet {
                    bet_type: bet_type.clone(),
                    amount: amount_u64,
                    player_id: 0,
                };
                let _ = game.place_bet(bet);
                self.state.single_player_game.set(game);
            }
            RouletteOperation::Spin {} => {
                let game = self.state.single_player_game.get_mut();
                let timestamp = self.runtime.system_time().to_string();
                let hash = format!("{:?}", self.runtime.chain_id());
                // Generate random number between 0 and 36
                let result = get_random_value(0, 37, hash, timestamp).unwrap_or(0) as u8;
                let _winnings = game.spin(result).unwrap_or_default();
                let game_clone = game.clone();
                let _ = game; // Release mutable borrow before set
                self.state.single_player_game.set(game_clone);
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

