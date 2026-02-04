use bankroll::{
    AccessControl, CasinoConfig, DailyBonus, DebtRecord, PlayerLimits, PlayerStats,
    PublicChainBalances, StakerInfo, StakingPool, TokenPotRecord,
};
use linera_sdk::linera_base_types::{AccountOwner, Amount, ChainId};
use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext};

#[derive(RootView, async_graphql::SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct BankrollState {
    // === All Chain ===
    pub casino_token: RegisterView<Amount>,
    pub debt_log: MapView<u64, DebtRecord>,
    
    // === Public Chain ===
    pub token_pot_log: MapView<u64, TokenPotRecord>,
    
    // === User Chain ===
    pub daily_bonus: RegisterView<DailyBonus>,
    pub accounts: MapView<AccountOwner, Amount>,
    
    // === Master Chain ===
    pub balances: MapView<ChainId, PublicChainBalances>,
    
    // === Staking System (NEW) ===
    /// Global staking pool state
    pub staking_pool: RegisterView<StakingPool>,
    /// Individual staker information
    pub stakers: MapView<AccountOwner, StakerInfo>,
    
    // === Responsible Gaming (NEW) ===
    /// Player-specific betting limits
    pub player_limits: MapView<AccountOwner, PlayerLimits>,
    /// Player lifetime statistics
    pub player_stats: MapView<AccountOwner, PlayerStats>,
    
    // === Governance (NEW) ===
    /// Access control (admin, operators)
    pub access_control: RegisterView<AccessControl>,
    /// Global casino configuration
    pub casino_config: RegisterView<CasinoConfig>,
}
