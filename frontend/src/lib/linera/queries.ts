import { gql } from '@apollo/client';

// Poker Queries
export const GET_POKER_SINGLE_PLAYER_DATA = gql`
  query GetPokerSinglePlayerData {
    singlePlayerData {
      userStatus
      game {
        players {
          id
          name
          holeCards
          chips
          currentBet
          isFolded
          isActive
        }
        communityCards
        pot
        currentRound
        status
      }
    }
  }
`;

export const GET_POKER_MULTI_PLAYER_DATA = gql`
  query GetPokerMultiPlayerData {
    multiPlayerData {
      userStatus
      game {
        players {
          id
          name
          holeCards
          chips
          currentBet
          isFolded
          isActive
        }
        communityCards
        pot
        currentRound
        status
      }
    }
  }
`;

export const GET_POKER_PROFILE = gql`
  query GetPokerProfile {
    getProfile {
      seat
      balance
      betData {
        minBet
        maxBet
        chipset {
          amount
          text
          enable
        }
      }
    }
  }
`;

// Rummy Queries
export const GET_RUMMY_SINGLE_PLAYER_DATA = gql`
  query GetRummySinglePlayerData {
    singlePlayerData {
      userStatus
      game {
        players {
          id
          name
          hand
          melds {
            cards
            meldType
          }
          chips
          hasDeclared
        }
        deck {
          cards
        }
        discardPile
        status
      }
    }
  }
`;

export const GET_RUMMY_PROFILE = gql`
  query GetRummyProfile {
    getProfile {
      seat
      balance
      betData {
        minBet
        maxBet
      }
    }
  }
`;

// Roulette Queries
export const GET_ROULETTE_SINGLE_PLAYER_DATA = gql`
  query GetRouletteSinglePlayerData {
    singlePlayerData {
      userStatus
      game {
        status
        currentNumber
        bets {
          betType
          amount
          playerId
        }
        history
        pot
      }
    }
  }
`;

export const GET_ROULETTE_PROFILE = gql`
  query GetRouletteProfile {
    getProfile {
      seat
      balance
      betData {
        minBet
        maxBet
        chipset {
          amount
          text
          enable
        }
      }
    }
  }
`;

// Bankroll Queries
export const GET_BALANCE = gql`
  query GetBalance {
    getBalances {
      chain
      amount
    }
  }
`;

