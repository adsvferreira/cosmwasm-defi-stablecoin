import { Contract, wasmKitTypes, Coin } from "@kubiklabs/wasmkit";
export type ExecuteMsg = {
    deposit_collateral_and_mint_dsc: {
        amount_collateral: Uint128;
        amount_dsc_to_mint: Uint128;
        collateral_asset: AssetInfoBase_for_Addr;
    };
} | {
    redeem_collateral_for_dsc: {
        amount_collateral: Uint128;
        amount_dsc_to_burn: Uint128;
        collateral_asset: AssetInfoBase_for_Addr;
    };
} | {
    redeem_collateral: {
        amount_collateral: Uint128;
        collateral_asset: AssetInfoBase_for_Addr;
    };
} | {
    burn_dsc: {
        amount_dsc_to_burn: Uint128;
    };
} | {
    liquidate: {
        collateral_asset: AssetInfoBase_for_Addr;
        debt_to_cover: Decimal;
        user: string;
    };
};
export type Uint128 = string;
export type AssetInfoBase_for_Addr = {
    native: string;
} | {
    cw20: Addr;
};
export type Addr = string;
export type Decimal = string;
export interface InstantiateMsg {
    assets: AssetInfoBase_for_Addr[];
    dsc_address: string;
    liquidation_bonus: Uint128;
    liquidation_threshold: Uint128;
    min_health_factor: Decimal;
    oracle_address: string;
    owner: string;
    price_feed_ids: string[];
    pyth_oracle_address: string;
}
export type QueryMsg = {
    config: {};
} | {
    collateral_balance_of_user: {
        collateral_asset: string;
        user: string;
    };
} | {
    user_health_factor: {
        user: string;
    };
} | {
    account_information: {
        user: string;
    };
} | {
    account_collateral_value_usd: {
        user: string;
    };
} | {
    calculate_health_factor: {
        collateral_value_usd: Decimal;
        total_dsc_minted: Uint128;
    };
} | {
    get_usd_value: {
        amount: Uint128;
        token: string;
    };
} | {
    get_token_amount_from_usd: {
        token: string;
        usd_amount: Decimal;
    };
} | {
    get_collateral_token_price_feed: {
        collateral_asset: string;
    };
} | {
    get_collateral_balance_of_user: {
        token: string;
        user: string;
    };
};
export interface DscEngineReadOnlyInterface {
    config: () => Promise<any>;
    collateralBalanceOfUser: ({ collateralAsset, user }: {
        collateralAsset: string;
        user: string;
    }) => Promise<any>;
    userHealthFactor: ({ user }: {
        user: string;
    }) => Promise<any>;
    accountInformation: ({ user }: {
        user: string;
    }) => Promise<any>;
    accountCollateralValueUsd: ({ user }: {
        user: string;
    }) => Promise<any>;
    calculateHealthFactor: ({ collateralValueUsd, totalDscMinted }: {
        collateralValueUsd: Decimal;
        totalDscMinted: Uint128;
    }) => Promise<any>;
    getUsdValue: ({ amount, token }: {
        amount: Uint128;
        token: string;
    }) => Promise<any>;
    getTokenAmountFromUsd: ({ token, usdAmount }: {
        token: string;
        usdAmount: Decimal;
    }) => Promise<any>;
    getCollateralTokenPriceFeed: ({ collateralAsset }: {
        collateralAsset: string;
    }) => Promise<any>;
    getCollateralBalanceOfUser: ({ token, user }: {
        token: string;
        user: string;
    }) => Promise<any>;
}
export declare class DscEngineQueryContract extends Contract implements DscEngineReadOnlyInterface {
    constructor(contractName: string, instantiateTag?: string);
    config: () => Promise<any>;
    collateralBalanceOfUser: ({ collateralAsset, user }: {
        collateralAsset: string;
        user: string;
    }) => Promise<any>;
    userHealthFactor: ({ user }: {
        user: string;
    }) => Promise<any>;
    accountInformation: ({ user }: {
        user: string;
    }) => Promise<any>;
    accountCollateralValueUsd: ({ user }: {
        user: string;
    }) => Promise<any>;
    calculateHealthFactor: ({ collateralValueUsd, totalDscMinted }: {
        collateralValueUsd: Decimal;
        totalDscMinted: Uint128;
    }) => Promise<any>;
    getUsdValue: ({ amount, token }: {
        amount: Uint128;
        token: string;
    }) => Promise<any>;
    getTokenAmountFromUsd: ({ token, usdAmount }: {
        token: string;
        usdAmount: Decimal;
    }) => Promise<any>;
    getCollateralTokenPriceFeed: ({ collateralAsset }: {
        collateralAsset: string;
    }) => Promise<any>;
    getCollateralBalanceOfUser: ({ token, user }: {
        token: string;
        user: string;
    }) => Promise<any>;
}
export interface DscEngineInterface extends DscEngineReadOnlyInterface {
    depositCollateralAndMintDsc: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amountCollateral, amountDscToMint, collateralAsset }: {
        amountCollateral: Uint128;
        amountDscToMint: Uint128;
        collateralAsset: AssetInfoBase_for_Addr;
    }) => Promise<any>;
    redeemCollateralForDsc: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amountCollateral, amountDscToBurn, collateralAsset }: {
        amountCollateral: Uint128;
        amountDscToBurn: Uint128;
        collateralAsset: AssetInfoBase_for_Addr;
    }) => Promise<any>;
    redeemCollateral: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amountCollateral, collateralAsset }: {
        amountCollateral: Uint128;
        collateralAsset: AssetInfoBase_for_Addr;
    }) => Promise<any>;
    burnDsc: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amountDscToBurn }: {
        amountDscToBurn: Uint128;
    }) => Promise<any>;
    liquidate: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { collateralAsset, debtToCover, user }: {
        collateralAsset: AssetInfoBase_for_Addr;
        debtToCover: Decimal;
        user: string;
    }) => Promise<any>;
}
export declare class DscEngineContract extends DscEngineQueryContract implements DscEngineInterface {
    constructor(instantiateTag?: string);
    depositCollateralAndMintDsc: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amountCollateral, amountDscToMint, collateralAsset }: {
        amountCollateral: Uint128;
        amountDscToMint: Uint128;
        collateralAsset: AssetInfoBase_for_Addr;
    }) => Promise<any>;
    redeemCollateralForDsc: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amountCollateral, amountDscToBurn, collateralAsset }: {
        amountCollateral: Uint128;
        amountDscToBurn: Uint128;
        collateralAsset: AssetInfoBase_for_Addr;
    }) => Promise<any>;
    redeemCollateral: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amountCollateral, collateralAsset }: {
        amountCollateral: Uint128;
        collateralAsset: AssetInfoBase_for_Addr;
    }) => Promise<any>;
    burnDsc: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amountDscToBurn }: {
        amountDscToBurn: Uint128;
    }) => Promise<any>;
    liquidate: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { collateralAsset, debtToCover, user }: {
        collateralAsset: AssetInfoBase_for_Addr;
        debtToCover: Decimal;
        user: string;
    }) => Promise<any>;
}
