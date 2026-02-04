// Copyright (c) Linera Casino
// SPDX-License-Identifier: Apache-2.0

//! Game History & Audit System
//!
//! This module provides immutable game records and audit logging for transparency.

use crate::provably_fair::RNGProof;
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::ChainId;
use serde::{Deserialize, Serialize};

scalar!(GameType);
/// Type of game for categorization
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum GameType {
    #[default]
    Poker = 0,
    Roulette = 1,
    Rummy = 2,
    Blackjack = 3,
    Tournament = 4,
}

impl GameType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameType::Poker => "poker",
            GameType::Roulette => "roulette",
            GameType::Rummy => "rummy",
            GameType::Blackjack => "blackjack",
            GameType::Tournament => "tournament",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "poker" => Some(GameType::Poker),
            "roulette" => Some(GameType::Roulette),
            "rummy" => Some(GameType::Rummy),
            "blackjack" => Some(GameType::Blackjack),
            "tournament" => Some(GameType::Tournament),
            _ => None,
        }
    }
}

/// Payout information for a winner
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct WinnerPayout {
    pub chain_id: ChainId,
    pub amount: u64,
    pub position: Option<u8>, // For tournaments
}

/// Immutable record of a completed game
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct GameRecord {
    /// Unique game identifier
    pub game_id: u64,
    /// Type of game
    pub game_type: GameType,
    /// Timestamp when game completed (microseconds)
    pub timestamp_micros: u64,
    /// Chain IDs of all players who participated
    pub players: Vec<ChainId>,
    /// SHA256 hash of the final game state
    pub outcome_hash: Vec<u8>,
    /// RNG proof for verification (if applicable)
    pub rng_proof: Option<RNGProof>,
    /// Total pot/wagered amount
    pub pot_size: u64,
    /// Payouts to winners
    pub winner_payouts: Vec<WinnerPayout>,
    /// House rake collected (if any)
    pub rake_collected: u64,
    /// Additional metadata as JSON string
    pub metadata: Option<String>,
}

impl GameRecord {
    /// Create a new game record
    pub fn new(
        game_id: u64,
        game_type: GameType,
        timestamp_micros: u64,
        players: Vec<ChainId>,
        pot_size: u64,
    ) -> Self {
        GameRecord {
            game_id,
            game_type,
            timestamp_micros,
            players,
            outcome_hash: Vec::new(),
            rng_proof: None,
            pot_size,
            winner_payouts: Vec::new(),
            rake_collected: 0,
            metadata: None,
        }
    }

    /// Set the outcome hash (computed from final game state)
    pub fn set_outcome_hash(&mut self, hash: Vec<u8>) {
        self.outcome_hash = hash;
    }

    /// Attach RNG proof for verification
    pub fn attach_rng_proof(&mut self, proof: RNGProof) {
        self.rng_proof = Some(proof);
    }

    /// Add a winner payout
    pub fn add_payout(&mut self, chain_id: ChainId, amount: u64, position: Option<u8>) {
        self.winner_payouts.push(WinnerPayout {
            chain_id,
            amount,
            position,
        });
    }

    /// Set rake collected
    pub fn set_rake(&mut self, rake: u64) {
        self.rake_collected = rake;
    }

    /// Verify the RNG proof if present
    pub fn verify_rng(&self) -> Option<bool> {
        self.rng_proof.as_ref().map(|proof| proof.verify())
    }
}

/// Player's individual game history entry
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct PlayerGameHistory {
    /// Reference to the game
    pub game_id: u64,
    /// Type of game
    pub game_type: GameType,
    /// Timestamp (microseconds)
    pub timestamp_micros: u64,
    /// Amount wagered
    pub wagered: u64,
    /// Amount won (before rake)
    pub won: u64,
    /// Net profit/loss
    pub profit: i64,
    /// Player's finishing position (for tournaments)
    pub position: Option<u8>,
}

impl PlayerGameHistory {
    pub fn new(
        game_id: u64,
        game_type: GameType,
        timestamp_micros: u64,
        wagered: u64,
        won: u64,
    ) -> Self {
        let profit = won as i64 - wagered as i64;
        PlayerGameHistory {
            game_id,
            game_type,
            timestamp_micros,
            wagered,
            won,
            profit,
            position: None,
        }
    }

    pub fn with_position(mut self, position: u8) -> Self {
        self.position = Some(position);
        self
    }
}

/// Audit log entry for administrative actions
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct AuditLogEntry {
    /// Unique log ID
    pub log_id: u64,
    /// Timestamp (microseconds)
    pub timestamp_micros: u64,
    /// Actor who performed the action
    pub actor_chain: ChainId,
    /// Type of action
    pub action_type: String,
    /// Description of the action
    pub description: String,
    /// Previous value (if applicable)
    pub previous_value: Option<String>,
    /// New value (if applicable)
    pub new_value: Option<String>,
}

impl AuditLogEntry {
    pub fn new(
        log_id: u64,
        timestamp_micros: u64,
        actor_chain: ChainId,
        action_type: String,
        description: String,
    ) -> Self {
        AuditLogEntry {
            log_id,
            timestamp_micros,
            actor_chain,
            action_type,
            description,
            previous_value: None,
            new_value: None,
        }
    }

    pub fn with_values(
        mut self,
        previous: Option<String>,
        new: Option<String>,
    ) -> Self {
        self.previous_value = previous;
        self.new_value = new;
        self
    }
}

/// Game type and count for statistics
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct GameTypeCount {
    pub game_type: String,
    pub count: u64,
}

/// Summary statistics for a player
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct PlayerStatsSummary {
    pub total_games: u64,
    pub total_wagered: u64,
    pub total_won: u64,
    pub total_profit: i64,
    pub games_by_type: Vec<GameTypeCount>,
    pub biggest_win: u64,
    pub biggest_loss: u64,
    pub win_rate_percent: u8,
}

impl PlayerStatsSummary {
    /// Update stats with a new game result
    pub fn update(&mut self, history: &PlayerGameHistory) {
        self.total_games += 1;
        self.total_wagered += history.wagered;
        self.total_won += history.won;
        self.total_profit += history.profit;

        if history.profit > 0 {
            self.biggest_win = self.biggest_win.max(history.profit as u64);
        } else {
            self.biggest_loss = self.biggest_loss.max((-history.profit) as u64);
        }

        // Update win rate
        if self.total_games > 0 {
            let wins = self.total_games / 2; // Placeholder - should track actual wins
            self.win_rate_percent = ((wins * 100) / self.total_games) as u8;
        }
    }
}
