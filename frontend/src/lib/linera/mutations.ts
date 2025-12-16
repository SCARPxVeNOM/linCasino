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

export const POKER_CREATE_LOBBY = gql`
  mutation PokerCreateLobby($maxPlayers: Int!) {
    createLobby(maxPlayers: $maxPlayers)
  }
`;

export const POKER_JOIN_LOBBY = gql`
  mutation PokerJoinLobby($lobbyId: String!, $name: String!) {
    joinLobby(lobbyId: $lobbyId, name: $name)
  }
`;

export const POKER_START_LOBBY = gql`
  mutation PokerStartLobby($lobbyId: String!) {
    startLobby(lobbyId: $lobbyId)
  }
`;

export const POKER_CREATE_TABLE = gql`
  mutation PokerCreateTable($smallBlind: String!, $bigBlind: String!, $maxPlayers: Int!) {
    createTable(smallBlind: $smallBlind, bigBlind: $bigBlind, maxPlayers: $maxPlayers)
  }
`;

export const POKER_SIT = gql`
  mutation PokerSit($tableChain: String!, $name: String!) {
    sit(tableChain: $tableChain, name: $name)
  }
`;

export const POKER_LEAVE = gql`
  mutation PokerLeave($tableChain: String!) {
    leave(tableChain: $tableChain)
  }
`;

export const POKER_PLAYER_ACTION = gql`
  mutation PokerPlayerAction(
    $tableChain: String!
    $handId: String!
    $seatId: Int!
    $action: String!
    $amount: String
  ) {
    playerAction(
      tableChain: $tableChain
      handId: $handId
      seatId: $seatId
      action: $action
      amount: $amount
    )
  }
`;

export const POKER_HEARTBEAT = gql`
  mutation PokerHeartbeat($tableChain: String!) {
    heartbeat(tableChain: $tableChain)
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

