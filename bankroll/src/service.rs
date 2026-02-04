#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Object, Schema};
use bankroll::{
    AccessControl, BankrollOperation, CasinoConfig, DailyBonus, PlayerLimits, PlayerStats,
    PublicChainBalances, StakerInfo, StakingPool,
};
use linera_sdk::linera_base_types::AccountOwner;
use linera_sdk::{graphql::GraphQLMutationRoot, linera_base_types::WithServiceAbi, views::View, Service, ServiceRuntime};

use self::state::BankrollState;

pub struct BankrollService {
    state: Arc<BankrollState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(BankrollService);

impl WithServiceAbi for BankrollService {
    type Abi = bankroll::BankrollAbi;
}

impl Service for BankrollService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = BankrollState::load(runtime.root_view_storage_context()).await.expect("Failed to load state");
        BankrollService {
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
            BankrollOperation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .finish()
        .execute(query)
        .await
    }
}

#[allow(dead_code)]
struct QueryRoot {
    state: Arc<BankrollState>,
    runtime: Arc<ServiceRuntime<BankrollService>>,
}

#[Object]
impl QueryRoot {
    async fn get_daily_bonus(&self) -> DailyBonus {
        self.state.daily_bonus.get().clone()
    }

    async fn get_balances(&self) -> Vec<PublicChainBalances> {
        let balances_keys = self.state.balances.indices().await.expect("Failed to read balances keys");
        let mut data = Vec::new();

        for key in balances_keys.into_iter() {
            let p = self.state.balances.get(&key).await.expect("Failed to get balances");
            data.push(p.expect("Failed to get balances"));
        }

        data
    }

    // === NEW QUERIES ===

    async fn get_staking_pool(&self) -> StakingPool {
        self.state.staking_pool.get().clone()
    }

    async fn get_staker_info(&self, owner: AccountOwner) -> StakerInfo {
        self.state.stakers.get(&owner).await.unwrap_or_default().unwrap_or_default()
    }

    async fn get_player_limits(&self, owner: AccountOwner) -> PlayerLimits {
        self.state.player_limits.get(&owner).await.unwrap_or_default().unwrap_or_default()
    }

    async fn get_player_stats(&self, owner: AccountOwner) -> PlayerStats {
        self.state.player_stats.get(&owner).await.unwrap_or_default().unwrap_or_default()
    }

    async fn get_casino_config(&self) -> CasinoConfig {
        self.state.casino_config.get().clone()
    }
    
    async fn get_access_control(&self) -> AccessControl {
        self.state.access_control.get().clone()
    }
}

