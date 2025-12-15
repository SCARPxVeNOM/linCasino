#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use self::state::RouletteState;
use abi::bet_chip_profile::Profile;
use abi::roulette::{GameData, UserStatus};
use async_graphql::{EmptySubscription, Object, Schema};
use linera_sdk::linera_base_types::ChainId;
use linera_sdk::{graphql::GraphQLMutationRoot, linera_base_types::WithServiceAbi, views::View, Service, ServiceRuntime};
use roulette::RouletteOperation;

pub struct RouletteService {
    state: Arc<RouletteState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(RouletteService);

impl WithServiceAbi for RouletteService {
    type Abi = roulette::RouletteAbi;
}

impl Service for RouletteService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = RouletteState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        RouletteService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, query: Self::Query) -> Self::QueryResponse {
        Schema::build(
            QueryRoot {
                state: self.state.clone(),
                runtime: self.runtime.clone(),
            },
            RouletteOperation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .finish()
        .execute(query)
        .await
    }
}

#[allow(dead_code)]
struct QueryRoot {
    state: Arc<RouletteState>,
    runtime: Arc<ServiceRuntime<RouletteService>>,
}

#[Object]
impl QueryRoot {
    async fn single_player_data(&self) -> GameData {
        GameData {
            user_status: self.state.user_status.get().clone(),
            game: Some(self.state.single_player_game.get().clone()),
        }
    }
    async fn multi_player_data(&self) -> GameData {
        GameData {
            user_status: self.state.user_status.get().clone(),
            game: Some(self.state.multi_player_game.get().clone()),
        }
    }
    async fn get_profile(&self) -> Profile {
        self.state.profile.get().clone()
    }
    async fn get_user_status(&self) -> UserStatus {
        self.state.user_status.get().clone()
    }
    async fn get_play_chains(&self) -> Vec<ChainId> {
        self.state.play_chain_status.indices().await.unwrap_or_default()
    }
}

