#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::RummyState;
use abi::rummy::{RummyGame, RummyPlayer, RummyStatus, UserStatus};
use abi::deck::{get_new_deck, Deck};
use bankroll::{BankrollOperation, BankrollResponse};
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};
use rummy::{RummyEvent, RummyMessage, RummyOperation, RummyParameters};

pub struct RummyContract {
    state: RummyState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(RummyContract);

impl WithContractAbi for RummyContract {
    type Abi = rummy::RummyAbi;
}

impl Contract for RummyContract {
    type Message = RummyMessage;
    type Parameters = RummyParameters;
    type InstantiationArgument = u64;
    type EventValue = RummyEvent;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = RummyState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        RummyContract { state, runtime }
    }

    async fn instantiate(&mut self, argument: Self::InstantiationArgument) {
        self.state.instantiate_value.set(argument);
        self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            RummyOperation::GetBalance {} => {
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
            RummyOperation::StartSinglePlayerGame { name } => {
                let mut game = RummyGame::new();
                let timestamp = self.runtime.system_time().to_string();
                let mut deck = Deck::with_cards(get_new_deck(timestamp.clone()));
                deck.shuffle(timestamp.clone(), timestamp);
                game.deck = deck;
                
                // Add player to game
                let profile = self.state.profile.get();
                // Convert Amount to u64 (Amount is in smallest unit, 1 token = 1e9 units)
                let balance_u128: u128 = profile.balance.saturating_div(1u128).into();
                let balance_u64: u64 = balance_u128.min(u64::MAX as u128) as u64;
                let player = RummyPlayer::new(0, name, balance_u64);
                game.add_player(player).unwrap_or_default();
                game.deal_cards().unwrap_or_default();
                
                game.status = RummyStatus::Playing;
                self.state.single_player_game.set(game);
                self.state.user_status.set(UserStatus::InSinglePlayerGame);
            }
            RummyOperation::DrawFromDeck {} => {
                let mut game = self.state.single_player_game.get().clone();
                let _ = game.draw_from_deck(0);
                self.state.single_player_game.set(game);
            }
            RummyOperation::DrawFromDiscard {} => {
                let mut game = self.state.single_player_game.get().clone();
                let _ = game.draw_from_discard(0);
                self.state.single_player_game.set(game);
            }
            RummyOperation::DiscardCard { card } => {
                let mut game = self.state.single_player_game.get().clone();
                let _ = game.discard_card(0, card);
                self.state.single_player_game.set(game);
            }
            RummyOperation::Declare {} => {
                let mut game = self.state.single_player_game.get().clone();
                if let Some(player) = game.players.get_mut(0) {
                    player.has_declared = true;
                }
                game.status = RummyStatus::Declared;
                self.state.single_player_game.set(game);
            }
            _ => {
                log::info!("Rummy operation not yet implemented: {:?}", operation);
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

