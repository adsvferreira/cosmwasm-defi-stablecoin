use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_asset::AssetInfo;
pub use cw_controllers::ClaimsResponse;
use std::collections::HashMap;

#[cw_serde]
pub struct InstantiateMsg {
    /// Address allowed to change contract parameters
    pub owner: String,
    /// AssetInfo of tokens that can be deposited and used as collateral
    pub assets: Vec<AssetInfo>,
    /// address of protocol wrapper for pyth oracles
    pub oracle_address: String,
    /// pyth oracle contract address
    /// https://docs.pyth.network/documentation/pythnet-price-feeds/cosmwasm
    pub pyth_oracle_address: String,
    /// pyth price feed id's for each asset that can be deposited and used as collateral
    /// https://pyth.network/developers/price-feed-ids#cosmwasm-stable
    pub price_feed_ids: Vec<String>,
    /// address of stable asset to be minted
    pub dsc_address: String,
    /// liquidation threshold = 50 means you need to be 200% over-collateralized
    pub liquidation_threshold: Uint128,
    /// liquidation_bonus = 10 means you get assets at a 10% discount when liquidating
    pub liquidation_bonus: Uint128,
    /// health factor that leads to liquidation
    pub min_health_factor: Decimal,
}

#[cw_serde]
pub enum ExecuteMsg {
    /*
     * @param collateral_asset: asset you're depositing as collateral
     * @param amount_collateral: The amount of collateral you're depositing
     * @param amount_dsc_to_mint: The amount of DSC you want to mint
     * @notice This function will deposit your collateral and mint DSC in one transaction
     */
    DepositCollateralAndMintDsc {
        collateral_asset: AssetInfo,
        amount_collateral: Uint128,
        amount_dsc_to_mint: Uint128,
    },
    /*
     * @param collateral_asset: asset deposited as collateral
     * @param amount_collateral: The amount of collateral you're depositing
     * @param amount_dsc_to_burn: The amount of DSC you want to burn
     * @notice This function will withdraw your collateral and burn DSC in one transaction
     */
    RedeemCollateralForDsc {
        collateral_asset: AssetInfo,
        amount_collateral: Uint128,
        amount_dsc_to_burn: Uint128,
    },
    /*
     * @param collateral_asset: The collateral asset you're redeeming
     * @param amount_collateral: The amount of collateral you're redeeming
     * @notice This function will redeem your collateral.
     * @notice If you have DSC minted, you will not be able to redeem until you burn your DSC
     */
    RedeemCollateral {
        collateral_asset: AssetInfo,
        amount_collateral: Uint128,
    },
    /*
     * @notice careful! You'll burn your DSC here! Make sure you want to do this...
     * @dev you might want to use this if you're nervous you might get liquidated and want to just burn
     * you DSC but keep your collateral in.
     */
    BurnDsc {
        amount_dsc_to_burn: Uint128,
    },
    /*
     * @param collateral_asset: The collateral asset you're using to make the protocol solvent again.
     * This is collateral that you're going to take from the user who is insolvent.
     * In return, you have to burn your DSC to pay off their debt, but you don't pay off your own.
     * @param user: The user who is insolvent. They have to have a _healthFactor below MIN_HEALTH_FACTOR
     * @param debt_to_cover: The amount of DSC you want to burn to cover the user's debt.
     *
     * @notice: You can partially liquidate a user.
     * @notice: You will get a 10% LIQUIDATION_BONUS for taking the users funds.
     * @notice: This function working assumes that the protocol will be roughly 150% overcollateralized in order for this to work.
     * @notice: A known bug would be if the protocol was only 100% collateralized, we wouldn't be able to liquidate anyone.
     * For example, if the price of the collateral plummeted before anyone could be liquidated.
     */
    Liquidate {
        collateral_asset: AssetInfo,
        user: String,
        debt_to_cover: Decimal,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(Uint128)]
    CollateralBalanceOfUser {
        user: String,
        collateral_asset: String,
    },
    #[returns(Decimal)]
    UserHealthFactor { user: String },
    #[returns(AccountInfoResponse)]
    AccountInformation { user: String },
    #[returns(Decimal)]
    AccountCollateralValueUsd { user: String },
    #[returns(Decimal)]
    CalculateHealthFactor {
        total_dsc_minted: Uint128,
        collateral_value_usd: Decimal,
    },
    #[returns(Decimal)]
    GetUsdValue { token: String, amount: Uint128 },
    #[returns(Uint128)]
    GetTokenAmountFromUsd { token: String, usd_amount: Decimal },
    #[returns(String)]
    GetCollateralTokenPriceFeed { collateral_asset: String },
}

#[cw_serde]
pub struct ConfigResponse {
    /// Address allowed to change contract parameters
    pub owner: Addr,
    /// List of depositable asset infos
    pub assets: Vec<AssetInfo>,
    /// key is asset deposited denom or address, value is price_feed_id (https://pyth.network/developers/price-feed-ids#cosmwasm-stable)
    pub assets_to_feeds: HashMap<String, String>,
    /// pyth oracle contract address
    /// https://docs.pyth.network/documentation/pythnet-price-feeds/cosmwasm
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

#[cw_serde]
pub struct AccountInfoResponse {
    /// Collaterals deposited by user in usd
    pub deposited_collateral_in_usd: Decimal,
    /// DCS Minted by user
    pub total_dsc_minted: Uint128,
}
