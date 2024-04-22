"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.DscEngineContract = exports.DscEngineQueryContract = void 0;
const wasmkit_1 = require("@kubiklabs/wasmkit");
class DscEngineQueryContract extends wasmkit_1.Contract {
    constructor(contractName, instantiateTag) {
        super(contractName, instantiateTag);
        this.config = async () => {
            return this.queryMsg({
                config: {}
            });
        };
        this.collateralBalanceOfUser = async ({ collateralAsset, user }) => {
            return this.queryMsg({
                collateral_balance_of_user: {
                    collateral_asset: collateralAsset,
                    user
                }
            });
        };
        this.userHealthFactor = async ({ user }) => {
            return this.queryMsg({
                user_health_factor: {
                    user
                }
            });
        };
        this.accountInformation = async ({ user }) => {
            return this.queryMsg({
                account_information: {
                    user
                }
            });
        };
        this.accountCollateralValueUsd = async ({ user }) => {
            return this.queryMsg({
                account_collateral_value_usd: {
                    user
                }
            });
        };
        this.calculateHealthFactor = async ({ collateralValueUsd, totalDscMinted }) => {
            return this.queryMsg({
                calculate_health_factor: {
                    collateral_value_usd: collateralValueUsd,
                    total_dsc_minted: totalDscMinted
                }
            });
        };
        this.getUsdValue = async ({ amount, token }) => {
            return this.queryMsg({
                get_usd_value: {
                    amount,
                    token
                }
            });
        };
        this.getTokenAmountFromUsd = async ({ token, usdAmount }) => {
            return this.queryMsg({
                get_token_amount_from_usd: {
                    token,
                    usd_amount: usdAmount
                }
            });
        };
        this.getCollateralTokenPriceFeed = async ({ collateralAsset }) => {
            return this.queryMsg({
                get_collateral_token_price_feed: {
                    collateral_asset: collateralAsset
                }
            });
        };
        this.getCollateralBalanceOfUser = async ({ token, user }) => {
            return this.queryMsg({
                get_collateral_balance_of_user: {
                    token,
                    user
                }
            });
        };
        this.config = this.config.bind(this);
        this.collateralBalanceOfUser = this.collateralBalanceOfUser.bind(this);
        this.userHealthFactor = this.userHealthFactor.bind(this);
        this.accountInformation = this.accountInformation.bind(this);
        this.accountCollateralValueUsd = this.accountCollateralValueUsd.bind(this);
        this.calculateHealthFactor = this.calculateHealthFactor.bind(this);
        this.getUsdValue = this.getUsdValue.bind(this);
        this.getTokenAmountFromUsd = this.getTokenAmountFromUsd.bind(this);
        this.getCollateralTokenPriceFeed = this.getCollateralTokenPriceFeed.bind(this);
        this.getCollateralBalanceOfUser = this.getCollateralBalanceOfUser.bind(this);
    }
}
exports.DscEngineQueryContract = DscEngineQueryContract;
class DscEngineContract extends DscEngineQueryContract {
    constructor(instantiateTag) {
        super("dsc_engine", instantiateTag);
        this.depositCollateralAndMintDsc = async ({ account, customFees, memo, transferAmount }, { amountCollateral, amountDscToMint, collateralAsset }) => {
            return await this.executeMsg({
                deposit_collateral_and_mint_dsc: {
                    amount_collateral: amountCollateral,
                    amount_dsc_to_mint: amountDscToMint,
                    collateral_asset: collateralAsset
                }
            }, account, customFees, memo, transferAmount);
        };
        this.redeemCollateralForDsc = async ({ account, customFees, memo, transferAmount }, { amountCollateral, amountDscToBurn, collateralAsset }) => {
            return await this.executeMsg({
                redeem_collateral_for_dsc: {
                    amount_collateral: amountCollateral,
                    amount_dsc_to_burn: amountDscToBurn,
                    collateral_asset: collateralAsset
                }
            }, account, customFees, memo, transferAmount);
        };
        this.redeemCollateral = async ({ account, customFees, memo, transferAmount }, { amountCollateral, collateralAsset }) => {
            return await this.executeMsg({
                redeem_collateral: {
                    amount_collateral: amountCollateral,
                    collateral_asset: collateralAsset
                }
            }, account, customFees, memo, transferAmount);
        };
        this.burnDsc = async ({ account, customFees, memo, transferAmount }, { amountDscToBurn }) => {
            return await this.executeMsg({
                burn_dsc: {
                    amount_dsc_to_burn: amountDscToBurn
                }
            }, account, customFees, memo, transferAmount);
        };
        this.liquidate = async ({ account, customFees, memo, transferAmount }, { collateralAsset, debtToCover, user }) => {
            return await this.executeMsg({
                liquidate: {
                    collateral_asset: collateralAsset,
                    debt_to_cover: debtToCover,
                    user
                }
            }, account, customFees, memo, transferAmount);
        };
        this.depositCollateralAndMintDsc = this.depositCollateralAndMintDsc.bind(this);
        this.redeemCollateralForDsc = this.redeemCollateralForDsc.bind(this);
        this.redeemCollateral = this.redeemCollateral.bind(this);
        this.burnDsc = this.burnDsc.bind(this);
        this.liquidate = this.liquidate.bind(this);
    }
}
exports.DscEngineContract = DscEngineContract;
//# sourceMappingURL=DscEngineContract.js.map