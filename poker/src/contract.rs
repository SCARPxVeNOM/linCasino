#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::PokerState;
use abi::poker::{PokerGame, PokerPlayer, PokerStatus, UserStatus};
use abi::deck::{get_new_deck, Deck};
use bankroll::{BankrollOperation, BankrollResponse};
use linera_sdk::{
    linera_base_types::WithContractAbi,
    views::{RootView, View},
    Contract, ContractRuntime,
};
use poker::{PokerEvent, PokerMessage, PokerOperation, PokerParameters};

pub struct PokerContract {
    state: PokerState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(PokerContract);

impl WithContractAbi for PokerContract {
    type Abi = poker::PokerAbi;
}

impl Contract for PokerContract {
    type Message = PokerMessage;
    type Parameters = PokerParameters;
    type InstantiationArgument = u64;
    type EventValue = PokerEvent;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = PokerState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        PokerContract { state, runtime }
    }

    async fn instantiate(&mut self, argument: Self::InstantiationArgument) {
        self.state.instantiate_value.set(argument);
        self.runtime.application_parameters();
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        match operation {
            PokerOperation::GetBalance {} => {
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
            PokerOperation::StartSinglePlayerGame { name } => {
                let mut game = PokerGame::new(10, 20);
                let timestamp = self.runtime.system_time().to_string();
                let mut deck = Deck::with_cards(get_new_deck(timestamp.clone()));
                deck.shuffle(timestamp.clone(), timestamp);
                game.deck = deck;
                
                // Add player to game
                let profile = self.state.profile.get();
                // Convert Amount to u64 (Amount is in smallest unit, 1 token = 1e9 units)
                let balance_u128: u128 = profile.balance.saturating_div(1u128).into();
                let balance_u64: u64 = balance_u128.min(u64::MAX as u128) as u64;
                let player = PokerPlayer::new(0, name, balance_u64);
                game.add_player(player).unwrap_or_default();
                game.deal_hole_cards().unwrap_or_default();
                
                game.status = PokerStatus::PreFlop;
                self.state.single_player_game.set(game);
                self.state.user_status.set(UserStatus::InSinglePlayerGame);
            }
            PokerOperation::Bet { amount } => {
                let mut game = self.state.single_player_game.get().clone();
                if let Some(player) = game.players.get_mut(0) {
                    // Convert Amount to u64 (Amount is in smallest unit, 1 token = 1e9 units)
                    let amount_u128: u128 = amount.saturating_div(1u128).into();
                    let bet_amount: u64 = amount_u128.min(u64::MAX as u128) as u64;
                    if player.chips >= bet_amount {
                        player.chips -= bet_amount;
                        player.current_bet += bet_amount;
                        game.pot += bet_amount;
                        game.current_bet = bet_amount.max(game.current_bet);
                    }
                }
                self.state.single_player_game.set(game);
            }
            PokerOperation::Fold {} => {
                let mut game = self.state.single_player_game.get().clone();
                if let Some(player) = game.players.get_mut(0) {
                    player.is_folded = true;
                    player.is_active = false;
                }
                self.state.single_player_game.set(game);
            }
            PokerOperation::Call {} => {
                let mut game = self.state.single_player_game.get().clone();
                if let Some(player) = game.players.get_mut(0) {
                    let call_amount = game.current_bet.saturating_sub(player.current_bet);
                    if player.chips >= call_amount {
                        player.chips -= call_amount;
                        player.current_bet += call_amount;
                        game.pot += call_amount;
                    }
                }
                self.state.single_player_game.set(game);
            }
            PokerOperation::Raise { amount } => {
                let mut game = self.state.single_player_game.get().clone();
                if let Some(player) = game.players.get_mut(0) {
                    // Convert Amount to u64 (Amount is in smallest unit, 1 token = 1e9 units)
                    let amount_u128: u128 = amount.saturating_div(1u128).into();
                    let raise_amount: u64 = amount_u128.min(u64::MAX as u128) as u64;
                    let total_needed = game.current_bet + raise_amount;
                    let to_call = total_needed.saturating_sub(player.current_bet);
                    if player.chips >= to_call {
                        player.chips -= to_call;
                        player.current_bet = total_needed;
                        game.pot += to_call;
                        game.current_bet = total_needed;
                    }
                }
                self.state.single_player_game.set(game);
            }
            _ => {
                log::info!("Poker operation not yet implemented: {:?}", operation);
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

