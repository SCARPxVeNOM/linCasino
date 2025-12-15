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
    Number(u8),    // Bet on specific number (0-36)
    Red,           // Bet on red
    Black,          // Bet on black
    Even,           // Bet on even numbers
    Odd,            // Bet on odd numbers
    Low,            // Bet on 1-18
    High,           // Bet on 19-36
    Dozen(u8),      // Bet on dozen (1, 2, or 3)
    Column(u8),     // Bet on column (1, 2, or 3)
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct Bet {
    pub bet_type: String, // Serialized BetType as string
    pub amount: u64,
    pub player_id: u8,
}

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct RouletteGame {
    pub status: RouletteStatus,
    pub current_number: Option<u8>,
    pub bets: Vec<Bet>,
    pub history: Vec<u8>,
    pub pot: u64,
}

impl RouletteGame {
    pub fn new() -> Self {
        RouletteGame {
            status: RouletteStatus::WaitingForBets,
            current_number: None,
            bets: vec![],
            history: vec![],
            pot: 0,
        }
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
    
    pub fn place_bet_with_type(&mut self, bet_type: BetType, amount: u64, player_id: u8) -> Result<(), String> {
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
        });
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

