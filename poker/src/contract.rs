#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::PokerState;
use abi::poker::{
    ActionKind, LobbyPlayerInfo, PokerGame, PokerLobby, PokerPlayer, PokerStatus, UserStatus,
    DEFAULT_ACTION_TIMEOUT_MICROS,
};
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
            PokerOperation::CreateLobby { max_players } => {
                // Only allow lobby creation from the configured master chain to keep things simple.
                let params = self.runtime.application_parameters();
                assert_eq!(
                    self.runtime.chain_id(),
                    params.master_chain,
                    "Lobbies can only be created on the master chain"
                );

                let now = self.runtime.system_time();
                let lobby_id = format!("{}-{}", self.runtime.chain_id(), now.micros());

                let lobby = PokerLobby {
                    id: lobby_id.clone(),
                    host_chain: self.runtime.chain_id(),
                    created_at_micros: now.micros(),
                    max_players,
                    started: false,
                    players: Vec::new(),
                };

                self.state
                    .lobbies
                    .insert(&lobby_id, lobby)
                    .unwrap_or_else(|_| panic!("Failed to create lobby {}", lobby_id));
            }
            PokerOperation::JoinLobby { lobby_id, name } => {
                // Join an existing lobby on the master/default chain.
                let params = self.runtime.application_parameters();
                assert_eq!(
                    self.runtime.chain_id(),
                    params.master_chain,
                    "Lobbies can only be joined on the master chain"
                );

                let chain_id = self.runtime.chain_id();
                if let Some(mut lobby) = self
                    .state
                    .lobbies
                    .get(&lobby_id)
                    .await
                    .expect("Failed to read lobby")
                {
                    if lobby.started {
                        // Game already started; ignore.
                    } else if lobby.players.len() >= lobby.max_players as usize {
                        // Lobby is full; ignore.
                    } else if lobby
                        .players
                        .iter()
                        .any(|p| p.chain_id == chain_id && p.name == name)
                    {
                        // Player already joined; nothing to do.
                    } else {
                        lobby.players.push(LobbyPlayerInfo { chain_id, name });
                        self.state
                            .lobbies
                            .insert(&lobby_id, lobby)
                            .unwrap_or_else(|_| panic!("Failed to update lobby {}", lobby_id));
                    }
                }
            }
            // Multiplayer table operations
            PokerOperation::CreateTable { small_blind, big_blind, max_players: _ } => {
                // Initialize the authoritative table on this (play) chain.
                let mut game = PokerGame::new(
                    amount_to_u64(small_blind),
                    amount_to_u64(big_blind),
                );
                game.status = PokerStatus::WaitingForPlayers;
                game.hand_id = 0;
                self.state.game.set(game);
            }
            PokerOperation::Sit { table_chain, name } => {
                // On the user chain: fetch current balance and forward a JoinTable
                // message to the play chain with a virtual buy-in equal to the balance.
                let owner = self
                    .runtime
                    .authenticated_signer()
                    .expect("No authenticated signer for Sit");
                let bankroll_app_id = self.runtime.application_parameters().bankroll;
                let balance_response =
                    self.runtime
                        .call_application(true, bankroll_app_id, &BankrollOperation::Balance { owner });
                let buy_in = match balance_response {
                    BankrollResponse::Balance(amount) => amount,
                    _ => linera_sdk::linera_base_types::Amount::ZERO,
                };

                let my_chain = self.runtime.chain_id();
                self.state.user_status.set(UserStatus::InMultiPlayerGame);

                self.runtime
                    .prepare_message(poker::PokerMessage::JoinTable {
                        user_chain: my_chain,
                        name,
                        buy_in,
                    })
                    .with_tracking()
                    .send_to(table_chain);
            }
            PokerOperation::Leave { table_chain } => {
                let my_chain = self.runtime.chain_id();
                self.state.user_status.set(UserStatus::Idle);
                self.runtime
                    .prepare_message(poker::PokerMessage::LeaveTable { user_chain: my_chain })
                    .with_tracking()
                    .send_to(table_chain);
            }
            PokerOperation::PlayerAction {
                table_chain,
                hand_id,
                seat_id,
                action,
                amount,
            } => {
                let my_chain = self.runtime.chain_id();
                self.runtime
                    .prepare_message(poker::PokerMessage::ApplyAction {
                        user_chain: my_chain,
                        hand_id,
                        seat_id,
                        action,
                        amount,
                    })
                    .with_tracking()
                    .send_to(table_chain);
            }
            PokerOperation::Heartbeat { table_chain } => {
                self.runtime
                    .prepare_message(poker::PokerMessage::Heartbeat)
                    .with_tracking()
                    .send_to(table_chain);
            }
            // Forward MintToken operations to the shared Bankroll application.
            // This is what the deployment script (`run.bash`) uses via GraphQL
            // and we also reuse it from the frontend "Get Tokens" button.
            PokerOperation::MintToken { chain_id, amount } => {
                let bankroll_app_id = self.runtime.application_parameters().bankroll;
                let _ = self.runtime.call_application(
                    true,
                    bankroll_app_id,
                    &BankrollOperation::MintToken { chain_id, amount },
                );
            }
            _ => {
                log::info!("Poker operation not yet implemented: {:?}", operation);
            }
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        let origin_chain_id = self
            .runtime
            .message_origin_chain_id()
            .expect("Chain ID missing from message");

        match message {
            // Existing protocol messages are currently unused in the new flow,
            // but we keep them for backward compatibility.
            PokerMessage::FindPlayChainResult { .. }
            | PokerMessage::RequestTableSeatResult { .. }
            | PokerMessage::ExitGameResult { .. }
            | PokerMessage::FindPlayChain
            | PokerMessage::AddPlayChain { .. }
            | PokerMessage::RegisterPublicChainAsPool { .. }
            | PokerMessage::RequestTableSeat { .. }
            | PokerMessage::AddingBet { .. }
            | PokerMessage::Fold { .. }
            | PokerMessage::Call { .. }
            | PokerMessage::Raise { .. }
            | PokerMessage::FindPlayChainSubscribe { .. }
            | PokerMessage::ExitGameRequest { .. } => {
                log::info!("Legacy PokerMessage received from {:?}", origin_chain_id);
            }

            // New multiplayer protocol messages
            PokerMessage::JoinTable {
                user_chain,
                name,
                buy_in,
            } => {
                self.handle_join_table(user_chain, name, buy_in).await;
            }
            PokerMessage::LeaveTable { user_chain } => {
                self.handle_leave_table(user_chain).await;
            }
            PokerMessage::ApplyAction {
                user_chain,
                hand_id,
                seat_id,
                action,
                amount,
            } => {
                self.handle_apply_action(origin_chain_id, user_chain, hand_id, seat_id, action, amount)
                    .await;
            }
            PokerMessage::Heartbeat => {
                self.handle_heartbeat().await;
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("Failed to save state");
    }
}

impl PokerContract {
    /// Seat a new player at the authoritative table hosted on this chain.
    async fn handle_join_table(
        &mut self,
        user_chain: linera_sdk::linera_base_types::ChainId,
        name: String,
        buy_in: linera_sdk::linera_base_types::Amount,
    ) {
        let mut game = self.state.game.get().clone();
        if game.players.len() >= abi::poker::MAX_POKER_PLAYERS {
            log::info!("Table full, cannot join more players");
            return;
        }

        let seat_id = game.players.len() as u8;
        let chips = amount_to_u64(buy_in);
        let mut player = PokerPlayer::new(seat_id, name, chips);
        player.is_active = true;
        game.players.push(player);

        // If we now have at least two players and are waiting, prepare first hand.
        if game.players.len() >= 2 && game.status == PokerStatus::WaitingForPlayers {
            game.hand_id = 1;
            game.dealer_position = 0;
            // First to act pre-flop: player after big blind (seat 2 in a 2+ player game).
            if let Some(cp) = game.next_active_player(game.dealer_position + 2) {
                game.current_player = Some(cp);
            }
            game.status = PokerStatus::PreFlop;
            let now = self.runtime.system_time();
            game.action_deadline_micros = now.micros() + DEFAULT_ACTION_TIMEOUT_MICROS;
        }

        self.state.game.set(game);
        log::info!("Player from chain {:?} joined table at seat {}", user_chain, seat_id);
    }

    async fn handle_leave_table(
        &mut self,
        _user_chain: linera_sdk::linera_base_types::ChainId,
    ) {
        // Simple implementation: no-op for now. In a full version we would
        // map user_chain to a seat and either mark the player as inactive or
        // remove them once not in an active hand and potentially cash out
        // via the bankroll application.
    }

    async fn handle_apply_action(
        &mut self,
        _origin_chain: linera_sdk::linera_base_types::ChainId,
        _user_chain: linera_sdk::linera_base_types::ChainId,
        hand_id: u64,
        seat_id: u8,
        action: ActionKind,
        amount: Option<linera_sdk::linera_base_types::Amount>,
    ) {
        let mut game = self.state.game.get().clone();

        // Hand/turn conflict checks
        if game.hand_id != hand_id {
            log::info!("Stale hand_id in ApplyAction, ignoring");
            return;
        }
        let Some(current_seat) = game.current_player else {
            log::info!("No current_player set, ignoring ApplyAction");
            return;
        };
        if current_seat != seat_id {
            log::info!("Seat {} tried to act out of turn (current is {})", seat_id, current_seat);
            return;
        }
        let Some(player) = game.players.get_mut(seat_id as usize) else {
            log::info!("Invalid seat_id {} in ApplyAction", seat_id);
            return;
        };

        if player.is_folded || !player.is_active || player.is_all_in {
            log::info!("Player at seat {} cannot act (folded/inactive/all-in)", seat_id);
            return;
        }

        let now_micros = self.runtime.system_time().micros();
        if now_micros > game.action_deadline_micros {
            log::info!("Action arrived after deadline, will be ignored");
            return;
        }

        let amount_u64 = amount.map(amount_to_u64).unwrap_or(0);

        match action {
            ActionKind::Fold => {
                player.is_folded = true;
                player.is_active = false;
            }
            ActionKind::Check => {
                if player.current_bet != game.current_bet {
                    log::info!("Illegal check (player bet {}, current_bet {})", player.current_bet, game.current_bet);
                    return;
                }
            }
            ActionKind::Call => {
                let to_call = game.current_bet.saturating_sub(player.current_bet);
                if to_call > 0 && player.chips >= to_call {
                    player.chips -= to_call;
                    player.current_bet += to_call;
                    game.pot += to_call;
                } else if to_call > 0 {
                    log::info!("Player cannot afford call of {}", to_call);
                    return;
                }
            }
            ActionKind::Bet => {
                if game.current_bet != 0 {
                    log::info!("Bet not allowed when current_bet is already {}", game.current_bet);
                    return;
                }
                if amount_u64 == 0 || amount_u64 > player.chips {
                    log::info!("Invalid bet amount {}", amount_u64);
                    return;
                }
                player.chips -= amount_u64;
                player.current_bet += amount_u64;
                game.pot += amount_u64;
                game.current_bet = amount_u64;
                game.min_raise = amount_u64;
            }
            ActionKind::Raise => {
                if amount_u64 < game.min_raise {
                    log::info!("Raise {} below min_raise {}", amount_u64, game.min_raise);
                    return;
                }
                let target = game.current_bet + amount_u64;
                let to_call = target.saturating_sub(player.current_bet);
                if to_call > player.chips {
                    log::info!("Player cannot afford raise, need {}", to_call);
                    return;
                }
                player.chips -= to_call;
                player.current_bet = target;
                game.pot += to_call;
                game.current_bet = target;
                game.min_raise = amount_u64;
            }
            ActionKind::AllIn => {
                let to_add = player.chips;
                if to_add == 0 {
                    log::info!("All-in with zero chips ignored");
                    return;
                }
                player.chips = 0;
                player.current_bet += to_add;
                game.pot += to_add;
                if player.current_bet > game.current_bet {
                    game.min_raise = player.current_bet - game.current_bet;
                    game.current_bet = player.current_bet;
                }
                player.is_all_in = true;
            }
        }

        // Move to next player or advance round.
        if game.is_betting_round_complete() {
            self.advance_round(&mut game);
        } else if let Some(next) = game.next_active_player(seat_id) {
            game.current_player = Some(next);
            game.action_deadline_micros = now_micros + DEFAULT_ACTION_TIMEOUT_MICROS;
        }

        self.state.game.set(game);
    }

    async fn handle_heartbeat(&mut self) {
        let mut game = self.state.game.get().clone();
        let now_micros = self.runtime.system_time().micros();

        if now_micros <= game.action_deadline_micros {
            return;
        }

        if let Some(seat_id) = game.current_player {
            if let Some(player) = game.players.get_mut(seat_id as usize) {
                // Simple timeout policy: fold if facing a bet, otherwise treat as check.
                if player.current_bet < game.current_bet {
                    player.is_folded = true;
                    player.is_active = false;
                }
            }

            if game.is_betting_round_complete() {
                self.advance_round(&mut game);
            } else if let Some(next) = game.next_active_player(seat_id) {
                game.current_player = Some(next);
                game.action_deadline_micros = now_micros + DEFAULT_ACTION_TIMEOUT_MICROS;
            }
        }

        self.state.game.set(game);
    }

    fn advance_round(&mut self, game: &mut PokerGame) {
        // Reset per-round bets
        for p in &mut game.players {
            p.current_bet = 0;
        }
        game.current_bet = 0;

        match game.current_round {
            abi::poker::BettingRound::PreFlop => {
                let _ = game.deal_flop();
                game.current_round = abi::poker::BettingRound::Flop;
            }
            abi::poker::BettingRound::Flop => {
                let _ = game.deal_turn();
                game.current_round = abi::poker::BettingRound::Turn;
            }
            abi::poker::BettingRound::Turn => {
                let _ = game.deal_river();
                game.current_round = abi::poker::BettingRound::River;
            }
            abi::poker::BettingRound::River | abi::poker::BettingRound::Showdown => {
                // Showdown and payout
                self.handle_showdown(game);
                game.status = PokerStatus::RoundEnded;
                game.hand_id = game.hand_id.saturating_add(1);
                game.community_cards.clear();
                game.pot = 0;
            }
        }

        // Choose next player to act for the new round.
        if let Some(cp) = game.current_player {
            if let Some(next) = game.next_active_player(cp) {
                game.current_player = Some(next);
            }
        }
        let now = self.runtime.system_time();
        game.action_deadline_micros = now.micros() + DEFAULT_ACTION_TIMEOUT_MICROS;
    }

    fn handle_showdown(&mut self, game: &mut PokerGame) {
        use abi::poker::evaluate_hand;

        let mut best_score = 0u32;
        let mut best_index: Option<usize> = None;

        for (idx, player) in game.players.iter().enumerate() {
            if player.is_folded {
                continue;
            }
            let (score, _) = evaluate_hand(&player.hole_cards, &game.community_cards);
            if score > best_score {
                best_score = score;
                best_index = Some(idx);
            }
        }

        if let Some(winner) = best_index {
            if let Some(player) = game.players.get_mut(winner) {
                player.chips = player.chips.saturating_add(game.pot);
            }
        }
    }
}

fn amount_to_u64(amount: linera_sdk::linera_base_types::Amount) -> u64 {
    let v: u128 = amount.saturating_div(1u128).into();
    v.min(u64::MAX as u128) as u64
}

