use abi::poker::PokerGame;
use async_graphql::{Request, Response};
use bankroll::BankrollAbi;
use linera_sdk::linera_base_types::{Amount, ApplicationId, ChainId};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PokerAbi;

impl ContractAbi for PokerAbi {
    type Operation = PokerOperation;
    type Response = ();
}

impl ServiceAbi for PokerAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum PokerOperation {
    // * User Chain
    SubscribeTo { chain_id: ChainId },
    UnsubscribeFrom { chain_id: ChainId },
    FindPlayChain {},
    ExitPlayChain {},
    ExitMultiPlayerGame {},
    RequestTableSeat { seat_id: u8, name: String },
    GetBalance {},
    Bet { amount: Amount },
    Fold {},
    Call {},
    Raise { amount: Amount },
    StartSinglePlayerGame { name: String },
    ExitSinglePlayerGame {},

    // * Master Chain
    AddPlayChain { target_public_chain: ChainId, play_chain_id: ChainId },
    MintToken { chain_id: ChainId, amount: Amount },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PokerMessage {
    // * User Chain
    FindPlayChainResult { latest_game_data: Option<PokerGame> },
    RequestTableSeatResult { name: String, seat_id: u8, success: bool },
    ExitGameResult { player_id: u8 },

    // * Public Chain
    FindPlayChain,
    AddPlayChain { chain_id: ChainId },

    // * Play Chain
    RegisterPublicChainAsPool { chain_id: ChainId },
    RequestTableSeat { seat_id: u8, balance: Amount, name: String },
    AddingBet { seat_id: u8, balance: Amount, amount: Amount },
    Fold { seat_id: u8 },
    Call { seat_id: u8, balance: Amount },
    Raise { seat_id: u8, balance: Amount, amount: Amount },
    FindPlayChainSubscribe { user_chain_id: ChainId },
    ExitGameRequest { seat_id: u8 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PokerParameters {
    pub master_chain: ChainId,
    pub public_chains: Vec<ChainId>,
    pub bankroll: ApplicationId<BankrollAbi>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PokerEvent {
    // * Event Subscriber
    GameState { game: PokerGame },
}

