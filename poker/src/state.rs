use abi::bet_chip_profile::Profile;
use abi::poker::{PokerGame, UserStatus};
use linera_sdk::linera_base_types::{Amount, ChainId};
use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext};

#[derive(RootView, async_graphql::SimpleObject)]
#[view(context = ViewStorageContext)]
pub struct PokerState {
    pub instantiate_value: RegisterView<u64>,
    // All Chain
    pub poker_token_pool: RegisterView<Amount>,
    pub token_pool_address: RegisterView<Option<ChainId>>,
    // Public Chain
    pub play_chain_set: MapView<u8, Vec<ChainId>>,
    pub play_chain_status: MapView<ChainId, u8>,
    // User Chain
    pub profile: RegisterView<Profile>,
    pub user_status: RegisterView<UserStatus>,
    pub user_play_chain: RegisterView<Option<ChainId>>,
    pub find_play_chain_retry: RegisterView<u8>,
    pub multi_player_game: RegisterView<PokerGame>,
    pub single_player_game: RegisterView<PokerGame>,
    // Play Chain
    pub game: RegisterView<PokerGame>,
}

