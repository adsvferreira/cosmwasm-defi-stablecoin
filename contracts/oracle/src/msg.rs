use std::time::Duration;

use pyth_sdk_cw::{Price, PriceIdentifier};

use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FetchPriceResponse)]
    FetchPrice {
        pyth_contract_addr: String,
        price_feed_id: PriceIdentifier,
    },
    #[returns(Duration)]
    FetchValidTimePeriod { pyth_contract_addr: String },
}

#[cw_serde]
pub struct FetchPriceResponse {
    pub current_price: Price,
    pub ema_price: Price,
}
