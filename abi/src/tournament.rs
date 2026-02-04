// Copyright (c) Linera Casino
// SPDX-License-Identifier: Apache-2.0

//! Tournament System
//!
//! This module provides tournament structures for poker and other games,
//! including registration, blind level management, and prize distribution.

use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::{Amount, ChainId};
use serde::{Deserialize, Serialize};

scalar!(TournamentStatus);
/// Current status of a tournament
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum TournamentStatus {
    #[default]
    Registration = 0,
    Running = 1,
    FinalTable = 2,
    Completed = 3,
    Cancelled = 4,
}

/// Blind level configuration
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct BlindLevel {
    /// Level number (1-indexed)
    pub level: u8,
    /// Small blind amount
    pub small_blind: u64,
    /// Big blind amount
    pub big_blind: u64,
    /// Ante amount (usually starts at 0)
    pub ante: u64,
    /// Duration of this level in microseconds
    pub duration_micros: u64,
}

impl BlindLevel {
    pub fn new(level: u8, small_blind: u64, big_blind: u64, ante: u64, duration_minutes: u64) -> Self {
        BlindLevel {
            level,
            small_blind,
            big_blind,
            ante,
            duration_micros: duration_minutes * 60 * 1_000_000,
        }
    }
}

/// Generate a standard blind schedule
pub fn generate_blind_schedule(
    starting_blind: u64,
    levels: u8,
    level_duration_minutes: u64,
) -> Vec<BlindLevel> {
    let mut schedule = Vec::with_capacity(levels as usize);
    let mut small_blind = starting_blind;
    
    for level in 1..=levels {
        let big_blind = small_blind * 2;
        // Ante starts at level 4
        let ante = if level >= 4 { small_blind / 2 } else { 0 };
        
        schedule.push(BlindLevel::new(
            level,
            small_blind,
            big_blind,
            ante,
            level_duration_minutes,
        ));
        
        // Increase blinds each level (roughly double every 2 levels)
        small_blind = if level % 2 == 0 {
            small_blind * 2
        } else {
            (small_blind * 3) / 2
        };
    }
    
    schedule
}

/// Tournament player information
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct TournamentPlayer {
    /// Player's chain ID
    pub chain_id: ChainId,
    /// Player's display name
    pub name: String,
    /// Current chip stack
    pub chips: u64,
    /// Table assignment (for multi-table tournaments)
    pub table_id: Option<u8>,
    /// Seat at the table
    pub seat_id: Option<u8>,
    /// Final position if eliminated (1 = winner)
    pub position: Option<u8>,
    /// Level at which player was eliminated
    pub eliminated_at_level: Option<u8>,
    /// Registration timestamp
    pub registered_at_micros: u64,
    /// Rebuys used (if allowed)
    pub rebuys: u8,
    /// Add-ons used (if allowed)
    pub addons: u8,
}

impl TournamentPlayer {
    pub fn new(chain_id: ChainId, name: String, starting_chips: u64, registered_at_micros: u64) -> Self {
        TournamentPlayer {
            chain_id,
            name,
            chips: starting_chips,
            table_id: None,
            seat_id: None,
            position: None,
            eliminated_at_level: None,
            registered_at_micros,
            rebuys: 0,
            addons: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.position.is_none() && self.chips > 0
    }

    pub fn eliminate(&mut self, position: u8, level: u8) {
        self.position = Some(position);
        self.eliminated_at_level = Some(level);
    }
}

/// Prize payout for a position
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct PrizePayout {
    pub position: u8,
    pub percentage: u8,
    pub amount: Amount,
    pub winner_chain: Option<ChainId>,
}

/// Main tournament structure
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct Tournament {
    /// Unique tournament ID
    pub id: u64,
    /// Tournament name
    pub name: String,
    /// Buy-in amount
    pub buy_in: Amount,
    /// Starting chip stack for each player
    pub starting_chips: u64,
    /// Maximum number of players
    pub max_players: u8,
    /// Minimum players required to start
    pub min_players: u8,
    /// Total prize pool (buy-ins + rebuys + add-ons)
    pub prize_pool: Amount,
    /// Current status
    pub status: TournamentStatus,
    /// Blind level schedule
    pub blind_schedule: Vec<BlindLevel>,
    /// Current blind level (0-indexed)
    pub current_level: u8,
    /// Timestamp when current level started
    pub level_start_time_micros: u64,
    /// Registered players
    pub players: Vec<TournamentPlayer>,
    /// Prize structure (percentages by position)
    pub prize_structure: Vec<u8>,
    /// Rebuy allowed until level
    pub rebuy_until_level: Option<u8>,
    /// Rebuy cost
    pub rebuy_cost: Option<Amount>,
    /// Rebuy chips received
    pub rebuy_chips: Option<u64>,
    /// Add-on available at level
    pub addon_at_level: Option<u8>,
    /// Add-on cost
    pub addon_cost: Option<Amount>,
    /// Add-on chips received
    pub addon_chips: Option<u64>,
    /// Tournament start time (scheduled)
    pub scheduled_start_micros: Option<u64>,
    /// Chain hosting this tournament
    pub host_chain: ChainId,
}

impl Tournament {
    /// Create a new tournament
    pub fn new(
        id: u64,
        name: String,
        buy_in: Amount,
        starting_chips: u64,
        max_players: u8,
        host_chain: ChainId,
    ) -> Self {
        Tournament {
            id,
            name,
            buy_in,
            starting_chips,
            max_players,
            min_players: 2,
            prize_pool: Amount::ZERO,
            status: TournamentStatus::Registration,
            blind_schedule: generate_blind_schedule(starting_chips / 100, 15, 10),
            current_level: 0,
            level_start_time_micros: 0,
            players: Vec::new(),
            prize_structure: vec![50, 30, 20], // Top 3 by default
            rebuy_until_level: None,
            rebuy_cost: None,
            rebuy_chips: None,
            addon_at_level: None,
            addon_cost: None,
            addon_chips: None,
            scheduled_start_micros: None,
            host_chain,
        }
    }

    /// Register a player for the tournament
    pub fn register(
        &mut self,
        chain_id: ChainId,
        name: String,
        timestamp_micros: u64,
    ) -> Result<(), String> {
        if self.status != TournamentStatus::Registration {
            return Err("Registration is closed".to_string());
        }

        if self.players.len() >= self.max_players as usize {
            return Err("Tournament is full".to_string());
        }

        if self.players.iter().any(|p| p.chain_id == chain_id) {
            return Err("Already registered".to_string());
        }

        self.players.push(TournamentPlayer::new(
            chain_id,
            name,
            self.starting_chips,
            timestamp_micros,
        ));
        
        self.prize_pool.saturating_add_assign(self.buy_in);
        
        Ok(())
    }

    /// Unregister a player (before tournament starts)
    pub fn unregister(&mut self, chain_id: ChainId) -> Result<Amount, String> {
        if self.status != TournamentStatus::Registration {
            return Err("Tournament has already started".to_string());
        }

        if let Some(pos) = self.players.iter().position(|p| p.chain_id == chain_id) {
            self.players.remove(pos);
            self.prize_pool = self.prize_pool.saturating_sub(self.buy_in);
            Ok(self.buy_in)
        } else {
            Err("Player not found".to_string())
        }
    }

    /// Start the tournament
    pub fn start(&mut self, current_time_micros: u64) -> Result<(), String> {
        if self.status != TournamentStatus::Registration {
            return Err("Tournament already started".to_string());
        }

        if self.players.len() < self.min_players as usize {
            return Err(format!("Need at least {} players", self.min_players));
        }

        self.status = TournamentStatus::Running;
        self.current_level = 0;
        self.level_start_time_micros = current_time_micros;
        
        Ok(())
    }

    /// Get current blind level
    pub fn get_current_blinds(&self) -> Option<&BlindLevel> {
        self.blind_schedule.get(self.current_level as usize)
    }

    /// Check if it's time to advance the blind level
    pub fn should_advance_level(&self, current_time_micros: u64) -> bool {
        if let Some(level) = self.get_current_blinds() {
            current_time_micros >= self.level_start_time_micros + level.duration_micros
        } else {
            false
        }
    }

    /// Advance to the next blind level
    pub fn advance_level(&mut self, current_time_micros: u64) -> bool {
        if self.current_level < self.blind_schedule.len() as u8 - 1 {
            self.current_level += 1;
            self.level_start_time_micros = current_time_micros;
            true
        } else {
            false
        }
    }

    /// Eliminate a player
    pub fn eliminate_player(&mut self, chain_id: ChainId) -> Result<u8, String> {
        let active_count = self.players.iter().filter(|p| p.is_active()).count();
        let position = active_count as u8; // Position = remaining players

        if let Some(player) = self.players.iter_mut().find(|p| p.chain_id == chain_id) {
            if player.position.is_some() {
                return Err("Player already eliminated".to_string());
            }
            player.eliminate(position, self.current_level);
            
            // Check if we're at final table (usually 9 or fewer)
            let remaining = self.players.iter().filter(|p| p.is_active()).count();
            if remaining <= 9 && self.status == TournamentStatus::Running {
                self.status = TournamentStatus::FinalTable;
            }
            
            // Check if tournament is complete
            if remaining <= 1 {
                self.status = TournamentStatus::Completed;
                // Mark the winner
                if let Some(winner) = self.players.iter_mut().find(|p| p.position.is_none()) {
                    winner.position = Some(1);
                }
            }
            
            Ok(position)
        } else {
            Err("Player not found".to_string())
        }
    }

    /// Process a rebuy
    pub fn rebuy(&mut self, chain_id: ChainId) -> Result<(), String> {
        let rebuy_level = self.rebuy_until_level.ok_or("Rebuys not allowed")?;
        let cost = self.rebuy_cost.ok_or("Rebuy cost not set")?;
        let chips = self.rebuy_chips.ok_or("Rebuy chips not set")?;

        if self.current_level > rebuy_level {
            return Err("Rebuy period has ended".to_string());
        }

        if let Some(player) = self.players.iter_mut().find(|p| p.chain_id == chain_id) {
            player.chips += chips;
            player.rebuys += 1;
            self.prize_pool.saturating_add_assign(cost);
            Ok(())
        } else {
            Err("Player not found".to_string())
        }
    }

    /// Calculate prize payouts
    pub fn calculate_prizes(&self) -> Vec<PrizePayout> {
        let mut payouts = Vec::new();
        
        for (i, &percentage) in self.prize_structure.iter().enumerate() {
            let position = (i + 1) as u8;
            let amount = self.prize_pool.saturating_mul(percentage as u128)
                .saturating_div(100);
            
            let winner = self.players.iter()
                .find(|p| p.position == Some(position))
                .map(|p| p.chain_id);
            
            payouts.push(PrizePayout {
                position,
                percentage,
                amount,
                winner_chain: winner,
            });
        }
        
        payouts
    }

    /// Get remaining players count
    pub fn remaining_players(&self) -> usize {
        self.players.iter().filter(|p| p.is_active()).count()
    }

    /// Get average stack
    pub fn average_stack(&self) -> u64 {
        let active: Vec<&TournamentPlayer> = self.players.iter()
            .filter(|p| p.is_active())
            .collect();
        
        if active.is_empty() {
            return 0;
        }
        
        let total: u64 = active.iter().map(|p| p.chips).sum();
        total / active.len() as u64
    }

    /// Get chip leader
    pub fn chip_leader(&self) -> Option<&TournamentPlayer> {
        self.players.iter()
            .filter(|p| p.is_active())
            .max_by_key(|p| p.chips)
    }
}

/// Tournament summary for display
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct TournamentSummary {
    pub id: u64,
    pub name: String,
    pub status: TournamentStatus,
    pub buy_in: Amount,
    pub prize_pool: Amount,
    pub registered_players: u8,
    pub max_players: u8,
    pub current_level: u8,
}

impl From<&Tournament> for TournamentSummary {
    fn from(t: &Tournament) -> Self {
        TournamentSummary {
            id: t.id,
            name: t.name.clone(),
            status: t.status.clone(),
            buy_in: t.buy_in,
            prize_pool: t.prize_pool,
            registered_players: t.players.len() as u8,
            max_players: t.max_players,
            current_level: t.current_level,
        }
    }
}
