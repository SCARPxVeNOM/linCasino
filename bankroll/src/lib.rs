use async_graphql::scalar;
use async_graphql::{InputObject, Request, Response, SimpleObject};
use linera_sdk::linera_base_types::{AccountOwner, Amount, ChainId, Timestamp};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BankrollAbi;

impl ContractAbi for BankrollAbi {
    type Operation = BankrollOperation;
    type Response = BankrollResponse;
}

impl ServiceAbi for BankrollAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum BankrollOperation {
    // === User Chain Operations ===
    Balance { owner: AccountOwner },
    UpdateBalance { owner: AccountOwner, amount: Amount },
    NotifyDebt { amount: Amount, target_chain: ChainId, game_type: String },
    TransferTokenPot { amount: Amount, target_chain: ChainId, game_type: String },
    
    // === Staking Operations (NEW) ===
    Stake { amount: Amount },
    Unstake { amount: Amount },
    ClaimStakingRewards,
    GetStakingInfo { owner: AccountOwner },
    
    // === Responsible Gaming Operations (NEW) ===
    SetDailyLossLimit { limit: Amount },
    SetMaxSingleBet { limit: Amount },
    SelfExclude { duration_days: u64 },
    RemoveSelfExclusion,
    GetPlayerLimits { owner: AccountOwner },
    
    // === Master Chain Operations ===
    MintToken { chain_id: ChainId, amount: Amount },
    
    // === Admin Operations (NEW) ===
    SetGlobalConfig { config: CasinoConfig },
    PauseGame { game_type: String },
    UnpauseGame { game_type: String },
    SetAdmin { new_admin: AccountOwner },
    AddOperator { operator: AccountOwner },
    RemoveOperator { operator: AccountOwner },
    DistributeStakingRewards,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum BankrollMessage {
    // * Public Chain
    TokenIssued { amount: Amount },
    DebtNotif { debt_id: u64, amount: Amount, created_at: Timestamp, game_type: String },
    TokenPot { amount: Amount, game_type: String },
    // * User Chain
    DebtPaid { debt_id: u64, amount: Amount, paid_at: Timestamp },
    // * Master Chain
    TokenUpdate { amount: Amount },
    // === Staking Messages (NEW) ===
    StakingRewardDistribution { amount_per_token: Amount },
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum BankrollResponse {
    #[default]
    Ok,
    Balance(Amount),
    StakingInfo(StakerInfo),
    PlayerLimits(PlayerLimits),
    Error(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BankrollParameters {
    pub master_chain: ChainId,
    pub bonus: Amount,
}

// === STAKING SYSTEM (NEW) ===

/// Information about a staker
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct StakerInfo {
    /// Account owner
    pub owner: Option<AccountOwner>,
    /// Amount currently staked
    pub staked_amount: Amount,
    /// Timestamp when stake was created/updated
    pub stake_timestamp: Timestamp,
    /// Unclaimed staking rewards
    pub unclaimed_rewards: Amount,
    /// Last reward claim timestamp
    pub last_reward_claim: Timestamp,
    /// Reward per token at time of stake (for accurate reward calculation)
    pub reward_per_token_paid: Amount,
}

impl StakerInfo {
    pub fn new(owner: AccountOwner, amount: Amount, timestamp: Timestamp) -> Self {
        StakerInfo {
            owner: Some(owner),
            staked_amount: amount,
            stake_timestamp: timestamp,
            unclaimed_rewards: Amount::ZERO,
            last_reward_claim: timestamp,
            reward_per_token_paid: Amount::ZERO,
        }
    }

    /// Calculate earned rewards based on global reward per token
    pub fn earned(&self, current_reward_per_token: Amount) -> Amount {
        let reward_diff = current_reward_per_token.saturating_sub(self.reward_per_token_paid);
        let earned = self.staked_amount.saturating_mul(reward_diff.into());
        self.unclaimed_rewards.saturating_add(earned)
    }
}

/// Global staking pool state
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct StakingPool {
    /// Total amount staked by all users
    pub total_staked: Amount,
    /// Total profit accumulated for distribution
    pub total_profit: Amount,
    /// Cumulative reward per token (scaled)
    pub reward_per_token: Amount,
    /// Last time rewards were distributed
    pub last_update_time: Timestamp,
    /// Number of active stakers
    pub staker_count: u64,
}

impl StakingPool {
    /// Add profit to be distributed to stakers
    pub fn add_profit(&mut self, amount: Amount) {
        self.total_profit.saturating_add_assign(amount);
        
        // Update reward per token
        if self.total_staked > Amount::ZERO {
            let reward_increment = amount.saturating_div(self.total_staked.into());
            self.reward_per_token.saturating_add_assign(reward_increment);
        }
    }
}

// === RESPONSIBLE GAMING (NEW) ===

/// Player-specific betting limits
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct PlayerLimits {
    /// Maximum loss allowed per day
    pub daily_loss_limit: Option<Amount>,
    /// Current losses today
    pub daily_losses: Amount,
    /// Last reset timestamp for daily losses
    pub last_loss_reset: Timestamp,
    /// Self-exclusion end timestamp (None = not excluded)
    pub self_exclusion_until: Option<Timestamp>,
    /// Maximum single bet amount
    pub max_single_bet: Option<Amount>,
}

impl PlayerLimits {
    /// Check if player can place a bet of given amount
    pub fn can_bet(&self, amount: Amount, current_time: Timestamp) -> Result<(), String> {
        // Check self-exclusion
        if let Some(exclusion_end) = self.self_exclusion_until {
            if current_time < exclusion_end {
                return Err("Self-exclusion is active".to_string());
            }
        }

        // Check max single bet
        if let Some(max) = self.max_single_bet {
            if amount > max {
                return Err(format!("Bet exceeds max single bet limit of {}", max));
            }
        }

        // Check daily loss limit
        if let Some(limit) = self.daily_loss_limit {
            if self.daily_losses.saturating_add(amount) > limit {
                return Err("Daily loss limit exceeded".to_string());
            }
        }

        Ok(())
    }

    /// Record a loss
    pub fn record_loss(&mut self, amount: Amount, current_time: Timestamp) {
        // Reset daily losses if it's a new day
        let day_micros = 24 * 60 * 60 * 1_000_000u64;
        if current_time.micros().saturating_sub(self.last_loss_reset.micros()) >= day_micros {
            self.daily_losses = Amount::ZERO;
            self.last_loss_reset = current_time;
        }
        
        self.daily_losses.saturating_add_assign(amount);
    }

    /// Set self-exclusion
    pub fn set_self_exclusion(&mut self, duration_days: u64, current_time: Timestamp) {
        let exclusion_micros = duration_days * 24 * 60 * 60 * 1_000_000;
        self.self_exclusion_until = Some(Timestamp::from(
            current_time.micros() + exclusion_micros
        ));
    }

    /// Remove self-exclusion (requires cooling off period has passed)
    pub fn remove_self_exclusion(&mut self, current_time: Timestamp) -> Result<(), String> {
        if let Some(exclusion_end) = self.self_exclusion_until {
            if current_time < exclusion_end {
                return Err("Cannot remove self-exclusion before it expires".to_string());
            }
        }
        self.self_exclusion_until = None;
        Ok(())
    }
}

// === VIP SYSTEM (NEW) ===

scalar!(VIPTier);
#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum VIPTier {
    #[default]
    Bronze = 0,
    Silver = 1,
    Gold = 2,
    Platinum = 3,
    Diamond = 4,
}

impl VIPTier {
    /// Get VIP tier based on lifetime wagered amount
    pub fn from_wagered(amount: Amount) -> Self {
        // Thresholds in tokens
        let amount_val: u128 = amount.into();
        match amount_val {
            0..=999 => VIPTier::Bronze,
            1000..=9999 => VIPTier::Silver,
            10000..=99999 => VIPTier::Gold,
            100000..=999999 => VIPTier::Platinum,
            _ => VIPTier::Diamond,
        }
    }

    /// Get bonus multiplier for this tier (in basis points, 10000 = 100%)
    pub fn bonus_multiplier(&self) -> u16 {
        match self {
            VIPTier::Bronze => 10000,   // 100% (no bonus)
            VIPTier::Silver => 10100,   // 101%
            VIPTier::Gold => 10250,     // 102.5%
            VIPTier::Platinum => 10500, // 105%
            VIPTier::Diamond => 11000,  // 110%
        }
    }
}

/// Player lifetime statistics for VIP tracking
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct PlayerStats {
    pub lifetime_wagered: Amount,
    pub lifetime_won: Amount,
    pub games_played: u64,
    pub vip_tier: VIPTier,
    pub biggest_win: Amount,
}

impl PlayerStats {
    /// Update stats after a game
    pub fn update(&mut self, wagered: Amount, won: Amount) {
        self.lifetime_wagered.saturating_add_assign(wagered);
        self.lifetime_won.saturating_add_assign(won);
        self.games_played += 1;
        if won > self.biggest_win {
            self.biggest_win = won;
        }
        // Recalculate VIP tier
        self.vip_tier = VIPTier::from_wagered(self.lifetime_wagered);
    }
}

// === GOVERNANCE (NEW) ===

/// Global casino configuration
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject, InputObject)]
#[graphql(input_name = "CasinoConfigInput")]
pub struct CasinoConfig {
    /// Poker rake percentage (0-100)
    pub poker_rake_percent: u8,
    /// Maximum poker rake per pot
    pub poker_rake_cap: Amount,
    /// Minimum stake amount for staking pool
    pub min_stake_amount: Amount,
    /// Minimum unstake amount
    pub min_unstake_amount: Amount,
    /// List of paused game types
    pub paused_games: Vec<String>,
    /// Whether the entire casino is paused
    pub is_paused: bool,
}

impl CasinoConfig {
    pub fn is_game_paused(&self, game_type: &str) -> bool {
        self.is_paused || self.paused_games.iter().any(|g| g == game_type)
    }
}

/// Access control for admin operations
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct AccessControl {
    pub admin: Option<AccountOwner>,
    pub operators: Vec<AccountOwner>,
}

impl AccessControl {
    pub fn is_admin(&self, owner: &AccountOwner) -> bool {
        self.admin.as_ref() == Some(owner)
    }
    
    pub fn is_operator(&self, owner: &AccountOwner) -> bool {
        self.is_admin(owner) || self.operators.contains(owner)
    }
    
    pub fn add_operator(&mut self, operator: AccountOwner) {
        if !self.operators.contains(&operator) {
            self.operators.push(operator);
        }
    }
    
    pub fn remove_operator(&mut self, operator: &AccountOwner) {
        self.operators.retain(|o| o != operator);
    }
}

// === EXISTING STRUCTURES ===

#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct DailyBonus {
    pub amount: Amount,
    pub last_claim: Timestamp,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct PublicChainBalances {
    pub chain: ChainId,
    pub amount: Amount,
}

impl DailyBonus {
    pub fn is_zero(&self) -> bool {
        self.amount == Amount::ZERO
    }
    pub fn update_bonus(&mut self, bonus: Amount) {
        if self.is_zero() {
            self.amount = bonus;
        }
    }
    pub fn claim_bonus(&mut self, current_time: Timestamp) -> Amount {
        let delta_since_last_claim = current_time.delta_since(self.last_claim).as_micros();
        if delta_since_last_claim >= ONE_DAY_CLAIM_DURATION_IN_MICROS {
            self.last_claim = current_time;
            return self.amount;
        }
        Amount::ZERO
    }
}

scalar!(DebtStatus);
#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize)]
#[repr(u8)]
pub enum DebtStatus {
    Pending = 0,
    Paid = 1,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct DebtRecord {
    pub id: u64,
    pub user_chain: ChainId,
    pub amount: Amount,
    pub created_at: Timestamp,
    pub paid_at: Option<Timestamp>,
    pub status: DebtStatus,
    pub game_type: String,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize, SimpleObject)]
pub struct TokenPotRecord {
    pub id: u64,
    pub user_chain: ChainId,
    pub amount: Amount,
    pub created_at: Timestamp,
    pub game_type: String,
}

const ONE_DAY_CLAIM_DURATION_IN_MICROS: u64 = 60 * 60 * 24 * 1_000_000;
