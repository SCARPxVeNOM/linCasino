use crate::deck::{get_card_rank, get_card_suit, Deck};
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use serde::{Deserialize, Serialize};

/// Maximum number of players allowed in a Poker game.
pub const MAX_POKER_PLAYERS: usize = 8;

/// The stream name the application uses for events about poker game event.
pub const POKER_STREAM_NAME: &[u8] = b"poker";

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
        }
    }
}

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
        }
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

