import { gql } from '@apollo/client';

// Poker Mutations
export const POKER_GET_BALANCE = gql`
  mutation PokerGetBalance {
    getBalance
  }
`;

export const POKER_START_SINGLE_PLAYER_GAME = gql`
  mutation PokerStartSinglePlayerGame($name: String!) {
    startSinglePlayerGame(name: $name)
  }
`;

export const POKER_BET = gql`
  mutation PokerBet($amount: String!) {
    bet(amount: $amount)
  }
`;

export const POKER_FOLD = gql`
  mutation PokerFold {
    fold
  }
`;

export const POKER_CALL = gql`
  mutation PokerCall {
    call
  }
`;

export const POKER_RAISE = gql`
  mutation PokerRaise($amount: String!) {
    raise(amount: $amount)
  }
`;

// Rummy Mutations
export const RUMMY_GET_BALANCE = gql`
  mutation RummyGetBalance {
    getBalance
  }
`;

export const RUMMY_START_SINGLE_PLAYER_GAME = gql`
  mutation RummyStartSinglePlayerGame($name: String!) {
    startSinglePlayerGame(name: $name)
  }
`;

export const RUMMY_DRAW_FROM_DECK = gql`
  mutation RummyDrawFromDeck {
    drawFromDeck
  }
`;

export const RUMMY_DRAW_FROM_DISCARD = gql`
  mutation RummyDrawFromDiscard {
    drawFromDiscard
  }
`;

export const RUMMY_DISCARD_CARD = gql`
  mutation RummyDiscardCard($card: Int!) {
    discardCard(card: $card)
  }
`;

export const RUMMY_DECLARE = gql`
  mutation RummyDeclare {
    declare
  }
`;

// Roulette Mutations
export const ROULETTE_GET_BALANCE = gql`
  mutation RouletteGetBalance {
    getBalance
  }
`;

export const ROULETTE_START_SINGLE_PLAYER_GAME = gql`
  mutation RouletteStartSinglePlayerGame($name: String!) {
    startSinglePlayerGame(name: $name)
  }
`;

export const ROULETTE_PLACE_BET = gql`
  mutation RoulettePlaceBet($betType: String!, $amount: String!) {
    placeBet(betType: $betType, amount: $amount)
  }
`;

export const ROULETTE_SPIN = gql`
  mutation RouletteSpin {
    spin
  }
`;

