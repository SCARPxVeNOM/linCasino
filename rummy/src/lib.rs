use abi::rummy::RummyGame;
use async_graphql::{Request, Response};
use bankroll::BankrollAbi;
use linera_sdk::linera_base_types::{Amount, ApplicationId, ChainId};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RummyAbi;

impl ContractAbi for RummyAbi {
    type Operation = RummyOperation;
    type Response = ();
}

impl ServiceAbi for RummyAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum RummyOperation {
    // * User Chain
    SubscribeTo { chain_id: ChainId },
    UnsubscribeFrom { chain_id: ChainId },
    FindPlayChain {},
    ExitPlayChain {},
    ExitMultiPlayerGame {},
    RequestTableSeat { seat_id: u8, name: String },
    GetBalance {},
    DrawFromDeck {},
    DrawFromDiscard {},
    DiscardCard { card: u8 },
    Declare {},
    StartSinglePlayerGame { name: String },
    ExitSinglePlayerGame {},

    // * Master Chain
    AddPlayChain { target_public_chain: ChainId, play_chain_id: ChainId },
    MintToken { chain_id: ChainId, amount: Amount },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RummyMessage {
    // * User Chain
    FindPlayChainResult { latest_game_data: Option<RummyGame> },
    RequestTableSeatResult { name: String, seat_id: u8, success: bool },
    ExitGameResult { player_id: u8 },

    // * Public Chain
    FindPlayChain,
    AddPlayChain { chain_id: ChainId },

    // * Play Chain
    RegisterPublicChainAsPool { chain_id: ChainId },
    RequestTableSeat { seat_id: u8, balance: Amount, name: String },
    DrawFromDeck { seat_id: u8 },
    DrawFromDiscard { seat_id: u8 },
    DiscardCard { seat_id: u8, card: u8 },
    Declare { seat_id: u8 },
    FindPlayChainSubscribe { user_chain_id: ChainId },
    ExitGameRequest { seat_id: u8 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RummyParameters {
    pub master_chain: ChainId,
    pub public_chains: Vec<ChainId>,
    pub bankroll: ApplicationId<BankrollAbi>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RummyEvent {
    GameState { game: RummyGame },
}

