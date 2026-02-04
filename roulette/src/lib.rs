use abi::roulette::RouletteGame;
use async_graphql::{Request, Response};
use bankroll::BankrollAbi;
use linera_sdk::linera_base_types::{Amount, ApplicationId, ChainId};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RouletteAbi;

impl ContractAbi for RouletteAbi {
    type Operation = RouletteOperation;
    type Response = ();
}

impl ServiceAbi for RouletteAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum RouletteOperation {
    // * User Chain
    SubscribeTo { chain_id: ChainId },
    UnsubscribeFrom { chain_id: ChainId },
    FindPlayChain {},
    ExitPlayChain {},
    ExitMultiPlayerGame {},
    RequestTableSeat { seat_id: u8, name: String },
    GetBalance {},
    PlaceBet { bet_type: String, amount: Amount },
    Spin {},
    StartSinglePlayerGame { name: String },
    ExitSinglePlayerGame {},
    /// Set client seed for provably fair spin
    SetClientSeed { seed: String },

    // * Master Chain
    AddPlayChain { target_public_chain: ChainId, play_chain_id: ChainId },
    MintToken { chain_id: ChainId, amount: Amount },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RouletteMessage {
    // * User Chain
    FindPlayChainResult { latest_game_data: Option<RouletteGame> },
    RequestTableSeatResult { name: String, seat_id: u8, success: bool },
    ExitGameResult { player_id: u8 },

    // * Public Chain
    FindPlayChain,
    AddPlayChain { chain_id: ChainId },

    // * Play Chain
    RegisterPublicChainAsPool { chain_id: ChainId },
    RequestTableSeat { seat_id: u8, balance: Amount, name: String },
    PlaceBet { seat_id: u8, bet_type: String, amount: Amount },
    Spin { seat_id: u8 },
    FindPlayChainSubscribe { user_chain_id: ChainId },
    ExitGameRequest { seat_id: u8 },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouletteParameters {
    pub master_chain: ChainId,
    pub public_chains: Vec<ChainId>,
    pub bankroll: ApplicationId<BankrollAbi>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RouletteEvent {
    GameState { game: RouletteGame },
}

