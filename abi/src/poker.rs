use crate::deck::{get_card_rank, get_card_suit, Deck};
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::ChainId;
use serde::{Deserialize, Serialize};

/// Maximum number of players allowed in a Poker game.
pub const MAX_POKER_PLAYERS: usize = 8;

/// The stream name the application uses for events about poker game event.
pub const POKER_STREAM_NAME: &[u8] = b"poker";

/// Default per-turn action timeout in microseconds (30 seconds).
pub const DEFAULT_ACTION_TIMEOUT_MICROS: u64 = 30_000_000;

scalar!(PokerStatus);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum PokerStatus {
    #[default]
    WaitingForPlayers = 0,
    PreFlop = 1,
    Flop = 2,
    Turn = 3,
    River = 4,
    Showdown = 5,
    RoundEnded = 6,
}

scalar!(BettingRound);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum BettingRound {
    #[default]
    PreFlop = 0,
    Flop = 1,
    Turn = 2,
    River = 3,
    Showdown = 4,
}

scalar!(UserStatus);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum UserStatus {
    #[default]
    Idle = 0,
    FindPlayChain = 1,
    PlayChainFound = 2,
    PlayChainUnavailable = 3,
    RequestingTableSeat = 4,
    RequestTableSeatFail = 5,
    InMultiPlayerGame = 6,
    InSinglePlayerGame = 7,
}

scalar!(ActionKind);
/// Player actions used in the multiplayer protocol.
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
pub enum ActionKind {
    Fold,
    Check,
    Call,
    Bet,
    Raise,
    AllIn,
}

scalar!(AutoAction);
/// Auto-actions for disconnected or away players
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
pub enum AutoAction {
    #[default]
    None,
    AutoFold,
    AutoCheckFold,
    AutoCallAny,
}

/// Session statistics for a player at the table
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct SessionStats {
    pub hands_played: u64,
    pub hands_won: u64,
    pub total_wagered: u64,
    pub total_won: u64,
    pub biggest_pot_won: u64,
}

impl SessionStats {
    pub fn record_hand(&mut self, wagered: u64, won: u64) {
        self.hands_played += 1;
        self.total_wagered += wagered;
        self.total_won += won;
        if won > 0 {
            self.hands_won += 1;
            self.biggest_pot_won = self.biggest_pot_won.max(won);
        }
    }

    pub fn profit(&self) -> i64 {
        self.total_won as i64 - self.total_wagered as i64
    }
}

/// Side pot for all-in scenarios
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct SidePot {
    /// Amount in this pot
    pub amount: u64,
    /// Player IDs eligible to win this pot
    pub eligible_players: Vec<u8>,
    /// The contribution level for this pot
    pub contribution_level: u64,
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct PokerPlayer {
    pub id: u8,
    pub name: String,
    pub hole_cards: Vec<u8>,
    pub chips: u64,
    pub current_bet: u64,
    pub is_folded: bool,
    pub is_active: bool,
    pub is_all_in: bool,
    /// Player is temporarily sitting out
    pub is_sitting_out: bool,
    /// Auto-action for when player's turn arrives
    pub auto_action: AutoAction,
    /// Session statistics
    pub session_stats: SessionStats,
    /// Player's chain ID for cross-chain messaging
    pub chain_id: Option<ChainId>,
}

impl PokerPlayer {
    pub fn new(id: u8, name: String, chips: u64) -> Self {
        PokerPlayer {
            id,
            name,
            hole_cards: vec![],
            chips,
            current_bet: 0,
            is_folded: false,
            is_active: true,
            is_all_in: false,
            is_sitting_out: false,
            auto_action: AutoAction::None,
            session_stats: SessionStats::default(),
            chain_id: None,
        }
    }

    pub fn with_chain_id(mut self, chain_id: ChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    pub fn set_auto_action(&mut self, action: AutoAction) {
        self.auto_action = action;
    }

    pub fn sit_out(&mut self) {
        self.is_sitting_out = true;
    }

    pub fn sit_in(&mut self) {
        self.is_sitting_out = false;
    }

    /// Check if player can act (not folded, not all-in, not sitting out)
    pub fn can_act(&self) -> bool {
        !self.is_folded && !self.is_all_in && self.is_active && !self.is_sitting_out
    }
}

/// Default rake percentage (5%)
pub const DEFAULT_RAKE_PERCENT: u8 = 5;
/// Default rake cap (500 units)
pub const DEFAULT_RAKE_CAP: u64 = 500;

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct PokerGame {
    pub players: Vec<PokerPlayer>,
    pub deck: Deck,
    pub community_cards: Vec<u8>,
    pub pot: u64,
    pub current_round: BettingRound,
    pub status: PokerStatus,
    pub dealer_position: u8,
    pub small_blind: u64,
    pub big_blind: u64,
    pub current_bet: u64,
    pub current_player: Option<u8>,
    /// Monotonically increasing identifier for each hand played at the table.
    pub hand_id: u64,
    /// Minimum amount required for a raise in the current betting round.
    pub min_raise: u64,
    /// Deadline (in microseconds since epoch) for the current player to act.
    pub action_deadline_micros: u64,
    /// Side pots for all-in scenarios
    pub side_pots: Vec<SidePot>,
    /// Total rake collected this session
    pub rake_collected: u64,
    /// Rake percentage (0-100)
    pub rake_percent: u8,
    /// Maximum rake per pot
    pub rake_cap: u64,
    /// RNG client seed for provably fair shuffle
    pub client_seed: Option<String>,
}

impl PokerGame {
    pub fn new(small_blind: u64, big_blind: u64) -> Self {
        PokerGame {
            players: vec![],
            deck: Deck::empty(),
            community_cards: vec![],
            pot: 0,
            current_round: BettingRound::PreFlop,
            status: PokerStatus::WaitingForPlayers,
            dealer_position: 0,
            small_blind,
            big_blind,
            current_bet: 0,
            current_player: None,
            hand_id: 0,
            min_raise: big_blind,
            action_deadline_micros: 0,
            side_pots: vec![],
            rake_collected: 0,
            rake_percent: DEFAULT_RAKE_PERCENT,
            rake_cap: DEFAULT_RAKE_CAP,
            client_seed: None,
        }
    }

    /// Create a game with custom rake settings
    pub fn with_rake(mut self, percent: u8, cap: u64) -> Self {
        self.rake_percent = percent;
        self.rake_cap = cap;
        self
    }

    /// Set client seed for provably fair RNG
    pub fn set_client_seed(&mut self, seed: String) {
        self.client_seed = Some(seed);
    }

    pub fn add_player(&mut self, player: PokerPlayer) -> Result<(), String> {
        if self.players.len() >= MAX_POKER_PLAYERS {
            return Err(format!("Maximum of {} players allowed in Poker.", MAX_POKER_PLAYERS));
        }
        self.players.push(player);
        Ok(())
    }

    pub fn remove_player(&mut self, player_id: u8) -> Result<(), String> {
        if let Some(pos) = self.players.iter().position(|p| p.id == player_id) {
            self.players.remove(pos);
            Ok(())
        } else {
            Err("Player not found".to_string())
        }
    }

    pub fn deal_hole_cards(&mut self) -> Result<(), String> {
        if self.players.len() < 2 {
            return Err("Need at least 2 players to start".to_string());
        }

        // Deal 2 cards to each player
        for _ in 0..2 {
            for player in &mut self.players {
                if let Some(card) = self.deck.deal_card() {
                    player.hole_cards.push(card);
                } else {
                    return Err("Not enough cards in deck".to_string());
                }
            }
        }
        Ok(())
    }

    pub fn deal_flop(&mut self) -> Result<(), String> {
        if self.community_cards.len() != 0 {
            return Err("Flop already dealt".to_string());
        }
        // Burn one card
        self.deck.deal_card();
        // Deal 3 community cards
        for _ in 0..3 {
            if let Some(card) = self.deck.deal_card() {
                self.community_cards.push(card);
            } else {
                return Err("Not enough cards in deck".to_string());
            }
        }
        self.current_round = BettingRound::Flop;
        Ok(())
    }

    pub fn deal_turn(&mut self) -> Result<(), String> {
        if self.community_cards.len() != 3 {
            return Err("Must deal flop first".to_string());
        }
        // Burn one card
        self.deck.deal_card();
        // Deal 1 community card
        if let Some(card) = self.deck.deal_card() {
            self.community_cards.push(card);
            self.current_round = BettingRound::Turn;
            Ok(())
        } else {
            Err("Not enough cards in deck".to_string())
        }
    }

    pub fn deal_river(&mut self) -> Result<(), String> {
        if self.community_cards.len() != 4 {
            return Err("Must deal turn first".to_string());
        }
        // Burn one card
        self.deck.deal_card();
        // Deal 1 community card
        if let Some(card) = self.deck.deal_card() {
            self.community_cards.push(card);
            self.current_round = BettingRound::River;
            Ok(())
        } else {
            Err("Not enough cards in deck".to_string())
        }
    }

    /// Find the next active player (not folded, not all-in) after `from`.
    pub fn next_active_player(&self, from: u8) -> Option<u8> {
        if self.players.is_empty() {
            return None;
        }
        let n = self.players.len() as u8;
        for step in 1..=n {
            let idx = (from + step) % n;
            if let Some(p) = self.players.get(idx as usize) {
                if !p.is_folded && p.is_active && !p.is_all_in {
                    return Some(idx);
                }
            }
        }
        None
    }

    /// Returns true if the betting round is complete (all active players have matched current_bet).
    pub fn is_betting_round_complete(&self) -> bool {
        let target = self.current_bet;
        let mut active_found = false;
        for p in &self.players {
            if p.is_folded || p.is_all_in || !p.is_active {
                continue;
            }
            active_found = true;
            if p.current_bet < target {
                return false;
            }
        }
        // If there are no active players, we consider the round complete.
        !active_found || true
    }

    /// Calculate side pots for all-in scenarios
    /// Call this when the hand is complete before distributing winnings
    pub fn calculate_side_pots(&mut self) {
        self.side_pots.clear();
        
        // Get all-in amounts sorted
        let mut contributions: Vec<(u8, u64)> = self.players
            .iter()
            .filter(|p| !p.is_folded && p.is_active)
            .map(|p| (p.id, p.current_bet))
            .collect();
        
        if contributions.is_empty() {
            return;
        }
        
        contributions.sort_by_key(|x| x.1);
        
        let mut prev_level = 0u64;
        let mut processed_ids: Vec<u8> = Vec::new();
        
        for (player_id, contribution) in &contributions {
            if *contribution > prev_level {
                let level_contribution = contribution - prev_level;
                
                // Eligible players are those who contributed at least this level
                let eligible: Vec<u8> = self.players
                    .iter()
                    .filter(|p| !p.is_folded && p.current_bet >= *contribution)
                    .map(|p| p.id)
                    .collect();
                
                // Count players contributing to this pot level
                let contributors = self.players
                    .iter()
                    .filter(|p| p.current_bet > prev_level)
                    .count() as u64;
                
                let pot_amount = level_contribution * contributors;
                
                if pot_amount > 0 && !eligible.is_empty() {
                    self.side_pots.push(SidePot {
                        amount: pot_amount,
                        eligible_players: eligible,
                        contribution_level: *contribution,
                    });
                }
                
                prev_level = *contribution;
            }
            processed_ids.push(*player_id);
        }
    }

    /// Calculate and collect rake from the pot
    /// Returns the rake amount collected
    pub fn collect_rake(&mut self) -> u64 {
        if self.rake_percent == 0 {
            return 0;
        }
        
        let rake = ((self.pot as u128 * self.rake_percent as u128) / 100) as u64;
        let rake = rake.min(self.rake_cap);
        
        if rake > 0 && self.pot >= rake {
            self.pot -= rake;
            self.rake_collected += rake;
        }
        
        rake
    }

    /// Get total rake collected this session
    pub fn total_rake(&self) -> u64 {
        self.rake_collected
    }

    /// Distribute pot to winner(s) after collecting rake
    /// Returns a list of (player_id, amount) for each winner
    pub fn distribute_pot(&mut self, winner_ids: &[u8]) -> Vec<(u8, u64)> {
        self.collect_rake();
        
        if winner_ids.is_empty() {
            return vec![];
        }
        
        let share = self.pot / winner_ids.len() as u64;
        let remainder = self.pot % winner_ids.len() as u64;
        
        let mut payouts: Vec<(u8, u64)> = Vec::new();
        
        for (i, &winner_id) in winner_ids.iter().enumerate() {
            // First winner gets any remainder
            let amount = if i == 0 { share + remainder } else { share };
            
            if let Some(player) = self.players.iter_mut().find(|p| p.id == winner_id) {
                player.chips += amount;
                player.session_stats.record_hand(player.current_bet, amount);
                payouts.push((winner_id, amount));
            }
        }
        
        self.pot = 0;
        payouts
    }

    /// Distribute side pots to winners based on hand strength
    /// hand_evaluator is a function that returns (score, [player_ids]) for best hands
    pub fn distribute_side_pots<F>(&mut self, mut evaluate_best: F) -> Vec<(u8, u64)>
    where
        F: FnMut(&[u8]) -> Vec<u8>, // Given eligible player IDs, return winner IDs
    {
        self.collect_rake();
        
        let mut all_payouts: Vec<(u8, u64)> = Vec::new();
        
        for pot in &self.side_pots {
            let winners = evaluate_best(&pot.eligible_players);
            if winners.is_empty() {
                continue;
            }
            
            let share = pot.amount / winners.len() as u64;
            let remainder = pot.amount % winners.len() as u64;
            
            for (i, &winner_id) in winners.iter().enumerate() {
                let amount = if i == 0 { share + remainder } else { share };
                all_payouts.push((winner_id, amount));
            }
        }
        
        // Apply payouts to player chips
        for (player_id, amount) in &all_payouts {
            if let Some(player) = self.players.iter_mut().find(|p| p.id == *player_id) {
                player.chips += amount;
            }
        }
        
        self.pot = 0;
        self.side_pots.clear();
        all_payouts
    }

    /// Reset for a new hand
    pub fn reset_for_new_hand(&mut self) {
        self.pot = 0;
        self.current_bet = 0;
        self.community_cards.clear();
        self.side_pots.clear();
        self.current_round = BettingRound::PreFlop;
        self.status = PokerStatus::WaitingForPlayers;
        self.min_raise = self.big_blind;
        self.hand_id += 1;
        
        for player in &mut self.players {
            player.hole_cards.clear();
            player.current_bet = 0;
            player.is_folded = false;
            player.is_all_in = false;
            // Auto-fold sitting out players
            if player.is_sitting_out {
                player.is_folded = true;
            }
        }
    }
}

/// Evaluate poker hand strength (simplified)
/// Returns a score where higher is better
pub fn evaluate_hand(hole_cards: &[u8], community_cards: &[u8]) -> (u32, String) {
    let all_cards = [hole_cards, community_cards].concat();
    if all_cards.len() < 5 {
        return (0, "Incomplete hand".to_string());
    }

    // Get ranks and suits
    let mut ranks: Vec<u8> = all_cards.iter().map(|&c| get_card_rank(c)).collect();
    let suits: Vec<u8> = all_cards.iter().map(|&c| get_card_suit(c)).collect();
    ranks.sort();

    // Check for flush
    let flush_suit = suits.iter().find(|&&s| {
        suits.iter().filter(|&&s2| s2 == s).count() >= 5
    });

    // Check for straight
    let mut straight_high = 0;
    let unique_ranks: Vec<u8> = ranks.iter().cloned().collect::<std::collections::HashSet<_>>().into_iter().collect();
    let mut sorted_unique = unique_ranks.clone();
    sorted_unique.sort();
    
    for i in 0..sorted_unique.len().saturating_sub(4) {
        if sorted_unique[i + 4] - sorted_unique[i] == 4 {
            straight_high = sorted_unique[i + 4];
            break;
        }
    }

    // Count rank frequencies
    let mut rank_counts: std::collections::HashMap<u8, u8> = std::collections::HashMap::new();
    for &rank in &ranks {
        *rank_counts.entry(rank).or_insert(0) += 1;
    }

    let mut counts: Vec<u8> = rank_counts.values().cloned().collect();
    counts.sort();
    counts.reverse();

    // Evaluate hand
    let is_straight_flush = flush_suit.is_some() && straight_high > 0;
    let is_four_of_kind = counts[0] == 4;
    let is_full_house = counts[0] == 3 && counts.len() > 1 && counts[1] >= 2;
    let is_flush = flush_suit.is_some();
    let is_straight = straight_high > 0;
    let is_three_of_kind = counts[0] == 3;
    let is_two_pair = counts[0] == 2 && counts.len() > 1 && counts[1] == 2;
    let is_pair = counts[0] == 2;

    if is_straight_flush {
        (900 + straight_high as u32, "Straight Flush".to_string())
    } else if is_four_of_kind {
        (800, "Four of a Kind".to_string())
    } else if is_full_house {
        (700, "Full House".to_string())
    } else if is_flush {
        (600, "Flush".to_string())
    } else if is_straight {
        (500 + straight_high as u32, "Straight".to_string())
    } else if is_three_of_kind {
        (400, "Three of a Kind".to_string())
    } else if is_two_pair {
        (300, "Two Pair".to_string())
    } else if is_pair {
        (200, "Pair".to_string())
    } else {
        (ranks[ranks.len() - 1] as u32, "High Card".to_string())
    }
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct GameData {
    pub user_status: UserStatus,
    pub game: Option<PokerGame>,
}

/// Information about a player waiting in a multiplayer lobby.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct LobbyPlayerInfo {
    pub chain_id: ChainId,
    pub name: String,
}

/// Simple multiplayer lobby representation.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct PokerLobby {
    /// Opaque lobby identifier that players can share.
    pub id: String,
    /// Chain on which the lobby was created (typically the master/default chain).
    pub host_chain: ChainId,
    /// Creation timestamp in microseconds.
    pub created_at_micros: u64,
    /// Maximum number of players that may join this lobby.
    pub max_players: u8,
    /// Whether the game has started already.
    pub started: bool,
    /// Players that have joined this lobby so far.
    pub players: Vec<LobbyPlayerInfo>,
}

