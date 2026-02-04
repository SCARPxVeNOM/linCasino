use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use serde::{Deserialize, Serialize};
use serde_json;

/// European Roulette numbers (0-36)
pub const ROULETTE_NUMBERS: [u8; 37] = [
    0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8, 23, 10, 5, 24, 16, 33, 1, 20, 14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26,
];

/// Red numbers on European Roulette
pub const RED_NUMBERS: [u8; 18] = [1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36];

/// The stream name the application uses for events about roulette game event.
pub const ROULETTE_STREAM_NAME: &[u8] = b"roulette";

/// Get neighbors of a number on the roulette wheel
/// Returns the center number plus `count` neighbors on each side
pub fn get_neighbors(center: u8, count: u8) -> Vec<u8> {
    // Find the position of center on the wheel
    let pos = ROULETTE_NUMBERS.iter().position(|&n| n == center);
    if pos.is_none() {
        return vec![center];
    }
    let pos = pos.unwrap();
    let len = ROULETTE_NUMBERS.len();
    
    let mut neighbors = Vec::with_capacity((count * 2 + 1) as usize);
    
    for i in 0..=(count * 2) {
        let offset = i as isize - count as isize;
        let idx = ((pos as isize + offset).rem_euclid(len as isize)) as usize;
        neighbors.push(ROULETTE_NUMBERS[idx]);
    }
    
    neighbors
}

/// Check if two numbers are adjacent on the roulette table layout
pub fn are_adjacent_on_table(a: u8, b: u8) -> bool {
    if a == 0 || b == 0 {
        // 0 is adjacent to 1, 2, 3
        return (a == 0 && b <= 3) || (b == 0 && a <= 3);
    }
    
    // Check horizontal adjacency (same row)
    let row_a = (a - 1) / 3;
    let row_b = (b - 1) / 3;
    let col_a = (a - 1) % 3;
    let col_b = (b - 1) % 3;
    
    // Same row, adjacent columns
    if row_a == row_b && (col_a as i8 - col_b as i8).abs() == 1 {
        return true;
    }
    
    // Same column, adjacent rows
    if col_a == col_b && (row_a as i8 - row_b as i8).abs() == 1 {
        return true;
    }
    
    false
}

scalar!(RouletteStatus);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum RouletteStatus {
    #[default]
    WaitingForBets = 0,
    Spinning = 1,
    Result = 2,
    RoundEnded = 3,
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

scalar!(BetType);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
pub enum BetType {
    // === Outside Bets ===
    Number(u8),     // Bet on specific number (0-36) - 35:1
    Red,            // Bet on red - 1:1
    Black,          // Bet on black - 1:1
    Even,           // Bet on even numbers - 1:1
    Odd,            // Bet on odd numbers - 1:1
    Low,            // Bet on 1-18 - 1:1
    High,           // Bet on 19-36 - 1:1
    Dozen(u8),      // Bet on dozen (1, 2, or 3) - 2:1
    Column(u8),     // Bet on column (1, 2, or 3) - 2:1
    
    // === Inside Bets (NEW) ===
    Split(u8, u8),              // Two adjacent numbers - 17:1
    Street(u8),                  // Row of 3 numbers (1-12 for rows 1-3, 4-6, etc.) - 11:1
    Corner(u8, u8, u8, u8),     // 4-number corner - 8:1
    Line(u8),                    // 6 numbers (two rows) - 5:1
    Basket,                      // 0, 1, 2, 3 (European) - 8:1
    
    // === Call Bets / Racetrack (NEW) ===
    Voisins,                     // Voisins du Zero (neighbors of zero) - 17 numbers
    Tiers,                       // Tiers du Cylindre (third of wheel) - 12 numbers
    Orphelins,                   // Orphelins (orphans) - 8 numbers
    Neighbors(u8, u8),          // Number + N neighbors on each side
    Zero,                        // Zero game (0, 3, 12, 15, 26, 32, 35)
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct Bet {
    pub bet_type: String, // Serialized BetType as string
    pub amount: u64,
    pub player_id: u8,
    /// Timestamp when bet was placed (microseconds)
    pub placed_at_micros: u64,
}

/// Default betting time in microseconds (30 seconds)
pub const DEFAULT_BETTING_TIME_MICROS: u64 = 30_000_000;

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct RouletteGame {
    pub status: RouletteStatus,
    pub current_number: Option<u8>,
    pub bets: Vec<Bet>,
    pub history: Vec<u8>,
    pub pot: u64,
    /// Betting deadline (microseconds since epoch)
    pub betting_deadline_micros: u64,
    /// Minimum bet amount
    pub min_bet: u64,
    /// Maximum bet amount per bet
    pub max_bet: u64,
    /// Maximum total bets per round
    pub table_limit: u64,
    /// Total amount bet this round
    pub total_bets_amount: u64,
    /// Round number
    pub round_number: u64,
    /// Client seed for provably fair RNG
    pub client_seed: Option<String>,
}

impl RouletteGame {
    pub fn new() -> Self {
        RouletteGame {
            status: RouletteStatus::WaitingForBets,
            current_number: None,
            bets: vec![],
            history: vec![],
            pot: 0,
            betting_deadline_micros: 0,
            min_bet: 1,
            max_bet: 10000,
            table_limit: 100000,
            total_bets_amount: 0,
            round_number: 0,
            client_seed: None,
        }
    }

    /// Create a game with custom limits
    pub fn with_limits(mut self, min_bet: u64, max_bet: u64, table_limit: u64) -> Self {
        self.min_bet = min_bet;
        self.max_bet = max_bet;
        self.table_limit = table_limit;
        self
    }

    /// Start the betting phase with a deadline
    pub fn start_betting(&mut self, current_time_micros: u64, duration_micros: u64) {
        self.status = RouletteStatus::WaitingForBets;
        self.betting_deadline_micros = current_time_micros + duration_micros;
        self.bets.clear();
        self.total_bets_amount = 0;
        self.round_number += 1;
    }

    /// Check if betting is still open
    pub fn is_betting_open(&self, current_time_micros: u64) -> bool {
        self.status == RouletteStatus::WaitingForBets
            && (self.betting_deadline_micros == 0 || current_time_micros < self.betting_deadline_micros)
    }

    /// Set client seed for provably fair RNG
    pub fn set_client_seed(&mut self, seed: String) {
        self.client_seed = Some(seed);
    }

    pub fn place_bet(&mut self, bet: Bet) -> Result<(), String> {
        if self.status != RouletteStatus::WaitingForBets {
            return Err("Bets are closed".to_string());
        }

        // Validate bet type (bet.bet_type is now a String, parse if needed)
        // For now, just accept the bet - validation can be done when deserializing

        self.bets.push(bet);
        Ok(())
    }
    
    pub fn place_bet_with_type(&mut self, bet_type: BetType, amount: u64, player_id: u8, current_time_micros: u64) -> Result<(), String> {
        if self.status != RouletteStatus::WaitingForBets {
            return Err("Bets are closed".to_string());
        }

        // Validate bet type
        match bet_type {
            BetType::Number(n) => {
                if n > 36 {
                    return Err("Invalid number".to_string());
                }
            }
            BetType::Dozen(d) => {
                if d < 1 || d > 3 {
                    return Err("Invalid dozen".to_string());
                }
            }
            BetType::Column(c) => {
                if c < 1 || c > 3 {
                    return Err("Invalid column".to_string());
                }
            }
            _ => {}
        }

        let bet_type_str = serde_json::to_string(&bet_type).unwrap_or_else(|_| "Unknown".to_string());
        self.bets.push(Bet {
            bet_type: bet_type_str,
            amount,
            player_id,
            placed_at_micros: current_time_micros,
        });
        self.total_bets_amount += amount;
        Ok(())
    }

    pub fn spin(&mut self, result: u8) -> Result<Vec<(u8, u64)>, String> {
        if self.status != RouletteStatus::WaitingForBets {
            return Err("Cannot spin now".to_string());
        }

        if result > 36 {
            return Err("Invalid result".to_string());
        }

        self.status = RouletteStatus::Spinning;
        self.current_number = Some(result);
        self.history.push(result);
        if self.history.len() > 10 {
            self.history.remove(0);
        }

        // Calculate winnings
        let mut winnings = vec![];
        for bet in &self.bets {
            // Parse bet_type string back to enum for calculation
            let bet_type: BetType = serde_json::from_str(&bet.bet_type).unwrap_or(BetType::Red);
            let payout = self.calculate_payout(&bet_type, result);
            if payout > 0 {
                winnings.push((bet.player_id, bet.amount * payout));
            }
        }

        self.status = RouletteStatus::Result;
        Ok(winnings)
    }

    pub fn calculate_payout(&self, bet_type: &BetType, result: u8) -> u64 {
        match bet_type {
            BetType::Number(n) => {
                if *n == result {
                    36 // 35:1 payout
                } else {
                    0
                }
            }
            BetType::Red => {
                if RED_NUMBERS.contains(&result) {
                    2 // 1:1 payout
                } else {
                    0
                }
            }
            BetType::Black => {
                if result != 0 && !RED_NUMBERS.contains(&result) {
                    2 // 1:1 payout
                } else {
                    0
                }
            }
            BetType::Even => {
                if result != 0 && result % 2 == 0 {
                    2 // 1:1 payout
                } else {
                    0
                }
            }
            BetType::Odd => {
                if result != 0 && result % 2 == 1 {
                    2 // 1:1 payout
                } else {
                    0
                }
            }
            BetType::Low => {
                if result >= 1 && result <= 18 {
                    2 // 1:1 payout
                } else {
                    0
                }
            }
            BetType::High => {
                if result >= 19 && result <= 36 {
                    2 // 1:1 payout
                } else {
                    0
                }
            }
            BetType::Dozen(d) => {
                let range = match d {
                    1 => 1..=12,
                    2 => 13..=24,
                    3 => 25..=36,
                    _ => return 0,
                };
                if range.contains(&result) {
                    3 // 2:1 payout
                } else {
                    0
                }
            }
            BetType::Column(c) => {
                // Column 1: 1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 31, 34
                // Column 2: 2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35
                // Column 3: 3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36
                let column_numbers = match c {
                    1 => [1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 31, 34],
                    2 => [2, 5, 8, 11, 14, 17, 20, 23, 26, 29, 32, 35],
                    3 => [3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36],
                    _ => return 0,
                };
                if column_numbers.contains(&result) {
                    3 // 2:1 payout
                } else {
                    0
                }
            }
            // === Inside Bets ===
            BetType::Split(a, b) => {
                if result == *a || result == *b {
                    18 // 17:1 payout
                } else {
                    0
                }
            }
            BetType::Street(row) => {
                // Street is a row of 3: row 1 = 1,2,3; row 2 = 4,5,6; etc.
                let start = (row - 1) * 3 + 1;
                if result >= start && result < start + 3 && result != 0 {
                    12 // 11:1 payout
                } else {
                    0
                }
            }
            BetType::Corner(a, b, c, d) => {
                if result == *a || result == *b || result == *c || result == *d {
                    9 // 8:1 payout
                } else {
                    0
                }
            }
            BetType::Line(row) => {
                // Line bet covers two rows = 6 numbers
                let start = (row - 1) * 3 + 1;
                if result >= start && result < start + 6 && result != 0 {
                    6 // 5:1 payout
                } else {
                    0
                }
            }
            BetType::Basket => {
                // European basket: 0, 1, 2, 3
                if result <= 3 {
                    9 // 8:1 payout (includes stake)
                } else {
                    0
                }
            }
            // === Call Bets ===
            BetType::Voisins => {
                // Voisins du Zero: 0, 2, 3, 4, 7, 12, 15, 18, 19, 21, 22, 25, 26, 28, 29, 32, 35
                const VOISINS: [u8; 17] = [0, 2, 3, 4, 7, 12, 15, 18, 19, 21, 22, 25, 26, 28, 29, 32, 35];
                if VOISINS.contains(&result) {
                    // Variable payout based on which number hit, simplified to 17:1 average
                    18
                } else {
                    0
                }
            }
            BetType::Tiers => {
                // Tiers du Cylindre: 5, 8, 10, 11, 13, 16, 23, 24, 27, 30, 33, 36
                const TIERS: [u8; 12] = [5, 8, 10, 11, 13, 16, 23, 24, 27, 30, 33, 36];
                if TIERS.contains(&result) {
                    18 // 17:1 payout
                } else {
                    0
                }
            }
            BetType::Orphelins => {
                // Orphelins: 1, 6, 9, 14, 17, 20, 31, 34
                const ORPHELINS: [u8; 8] = [1, 6, 9, 14, 17, 20, 31, 34];
                if ORPHELINS.contains(&result) {
                    18 // 17:1 payout
                } else {
                    0
                }
            }
            BetType::Neighbors(center, count) => {
                // Find neighbors on the wheel
                let neighbors = get_neighbors(*center, *count);
                if neighbors.contains(&result) {
                    36 / neighbors.len() as u64 // Split the 35:1 across all numbers
                } else {
                    0
                }
            }
            BetType::Zero => {
                // Zero game: 0, 3, 12, 15, 26, 32, 35
                const ZERO_GAME: [u8; 7] = [0, 3, 12, 15, 26, 32, 35];
                if ZERO_GAME.contains(&result) {
                    18 // 17:1 average payout
                } else {
                    0
                }
            }
        }
    }

    pub fn clear_bets(&mut self) {
        self.bets.clear();
        self.status = RouletteStatus::WaitingForBets;
        self.current_number = None;
    }
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct GameData {
    pub user_status: UserStatus,
    pub game: Option<RouletteGame>,
}

