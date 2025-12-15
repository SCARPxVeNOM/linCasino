use crate::deck::{get_card_rank, get_card_suit, Deck};
use async_graphql::scalar;
use async_graphql_derive::SimpleObject;
use serde::{Deserialize, Serialize};

/// Maximum number of players allowed in a Rummy game.
pub const MAX_RUMMY_PLAYERS: usize = 6;

/// The stream name the application uses for events about rummy game event.
pub const RUMMY_STREAM_NAME: &[u8] = b"rummy";

scalar!(RummyStatus);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum RummyStatus {
    #[default]
    WaitingForPlayers = 0,
    Dealing = 1,
    Playing = 2,
    Declared = 3,
    RoundEnded = 4,
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
pub struct RummyPlayer {
    pub id: u8,
    pub name: String,
    pub hand: Vec<u8>,
    pub melds: Vec<Meld>,
    pub chips: u64,
    pub has_declared: bool,
    pub is_active: bool,
}

impl RummyPlayer {
    pub fn new(id: u8, name: String, chips: u64) -> Self {
        RummyPlayer {
            id,
            name,
            hand: vec![],
            melds: vec![],
            chips,
            has_declared: false,
            is_active: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct Meld {
    pub cards: Vec<u8>,
    pub meld_type: String, // "Set" or "Sequence"
}

scalar!(MeldType);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
pub enum MeldType {
    Set,      // Three or more cards of the same rank
    Sequence, // Three or more consecutive cards of the same suit
}

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct RummyGame {
    pub players: Vec<RummyPlayer>,
    pub deck: Deck,
    pub discard_pile: Vec<u8>,
    pub status: RummyStatus,
    pub current_player: Option<u8>,
    pub pot: u64,
}

impl RummyGame {
    pub fn new() -> Self {
        RummyGame {
            players: vec![],
            deck: Deck::empty(),
            discard_pile: vec![],
            status: RummyStatus::WaitingForPlayers,
            current_player: None,
            pot: 0,
        }
    }

    pub fn add_player(&mut self, player: RummyPlayer) -> Result<(), String> {
        if self.players.len() >= MAX_RUMMY_PLAYERS {
            return Err(format!("Maximum of {} players allowed in Rummy.", MAX_RUMMY_PLAYERS));
        }
        self.players.push(player);
        Ok(())
    }

    pub fn deal_cards(&mut self) -> Result<(), String> {
        if self.players.len() < 2 {
            return Err("Need at least 2 players to start".to_string());
        }

        // Deal 13 cards to each player
        for _ in 0..13 {
            for player in &mut self.players {
                if let Some(card) = self.deck.deal_card() {
                    player.hand.push(card);
                } else {
                    return Err("Not enough cards in deck".to_string());
                }
            }
        }

        // Deal one card to discard pile
        if let Some(card) = self.deck.deal_card() {
            self.discard_pile.push(card);
        }

        self.status = RummyStatus::Playing;
        self.current_player = Some(0);
        Ok(())
    }

    pub fn draw_from_deck(&mut self, player_id: u8) -> Result<u8, String> {
        if let Some(card) = self.deck.deal_card() {
            if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
                player.hand.push(card);
                Ok(card)
            } else {
                Err("Player not found".to_string())
            }
        } else {
            Err("Deck is empty".to_string())
        }
    }

    pub fn draw_from_discard(&mut self, player_id: u8) -> Result<u8, String> {
        if let Some(card) = self.discard_pile.pop() {
            if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
                player.hand.push(card);
                Ok(card)
            } else {
                Err("Player not found".to_string())
            }
        } else {
            Err("Discard pile is empty".to_string())
        }
    }

    pub fn discard_card(&mut self, player_id: u8, card: u8) -> Result<(), String> {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if let Some(pos) = player.hand.iter().position(|&c| c == card) {
                player.hand.remove(pos);
                self.discard_pile.push(card);
                Ok(())
            } else {
                Err("Card not in player's hand".to_string())
            }
        } else {
            Err("Player not found".to_string())
        }
    }
    
    pub fn create_meld(&mut self, player_id: u8, cards: Vec<u8>, meld_type: MeldType) -> Result<(), String> {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            let meld_type_str = match meld_type {
                MeldType::Set => "Set",
                MeldType::Sequence => "Sequence",
            };
            player.melds.push(Meld {
                cards,
                meld_type: meld_type_str.to_string(),
            });
            Ok(())
        } else {
            Err("Player not found".to_string())
        }
    }
}

/// Validate if cards form a valid set (three or more cards of the same rank)
pub fn is_valid_set(cards: &[u8]) -> bool {
    if cards.len() < 3 {
        return false;
    }
    let ranks: Vec<u8> = cards.iter().map(|&c| get_card_rank(c)).collect();
    let first_rank = ranks[0];
    ranks.iter().all(|&r| r == first_rank)
}

/// Validate if cards form a valid sequence (three or more consecutive cards of the same suit)
pub fn is_valid_sequence(cards: &[u8]) -> bool {
    if cards.len() < 3 {
        return false;
    }
    let mut card_data: Vec<(u8, u8)> = cards.iter().map(|&c| (get_card_rank(c), get_card_suit(c))).collect();
    card_data.sort();
    
    let first_suit = card_data[0].1;
    if !card_data.iter().all(|(_, suit)| *suit == first_suit) {
        return false;
    }

    // Check consecutive ranks (handle Ace as both 1 and 14)
    for i in 1..card_data.len() {
        let prev_rank = card_data[i - 1].0;
        let curr_rank = card_data[i].0;
        if curr_rank != prev_rank + 1 && !(prev_rank == 13 && curr_rank == 1) {
            return false;
        }
    }
    true
}

/// Calculate deadwood (unmatched cards) points
pub fn calculate_deadwood(hand: &[u8], melds: &[Meld]) -> u32 {
    let mut used_cards = std::collections::HashSet::new();
    for meld in melds {
        for &card in &meld.cards {
            used_cards.insert(card);
        }
    }

    let mut deadwood = 0u32;
    for &card in hand {
        if !used_cards.contains(&card) {
            let rank = get_card_rank(card);
            deadwood += match rank {
                1 => 10,  // Ace
                11 | 12 | 13 => 10, // Face cards
                n => n as u32,
            };
        }
    }
    deadwood
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct GameData {
    pub user_status: UserStatus,
    pub game: Option<RummyGame>,
}

