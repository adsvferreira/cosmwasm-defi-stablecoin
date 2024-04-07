use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map, SnapshotMap};
use std::collections::HashMap;

/// This structure holds the main contract parameters.
#[cw_serde]
pub struct Config {
    /// Address allowed to change contract parameters
    pub owner: Addr,
    /// list of depositable asset infos
    pub assets: Vec<AssetInfo>,
    /// key is asset deposited denom or address, value is price_feed_id (https://pyth.network/developers/price-feed-ids#cosmwasm-stable)
    pub assets_to_feeds: HashMap<String, String>,
    /// address of protocol wrapper for pyth oracles
    pub oracle_address: Addr,
    /// pyth oracle contract address
    /// https://docs.pyth.network/documentation/pythnet-price-feeds/cosmwasm
    pub pyth_oracle_address: Addr,
    /// address of stable asset to be minted
    pub dsc_address: Addr,
    /// liquidation threshold = 50 means you need to be 200% over-collateralized
    pub liquidation_threshold: Uint128,
    /// liquidation_bonus = 10 means you get assets at a 10% discount when liquidating
    pub liquidation_bonus: Uint128,
    /// health factor that leads to liquidation
    pub min_health_factor: Decimal,
}

/// Saves dsc-engine settings
pub const CONFIG: Item<Config> = Item::new("config");

/// First key is user address, second key is collateral token denom/ address
pub const COLLATERAL_DEPOSITED: Map<(&Addr, String), Uint128> = Map::new("collateral_deposited");

pub const DSC_MINTED: SnapshotMap<&Addr, Uint128> = SnapshotMap::new(
    "balances",
    "balances_check",
    "balances_change",
    cw_storage_plus::Strategy::EveryBlock,
);
