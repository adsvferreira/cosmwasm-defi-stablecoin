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
  collateralBalanceOfUser: ({
    collateralAsset,
    user
  }: {
    collateralAsset: string;
    user: string;
  }) => Promise<any>;
  userHealthFactor: ({
    user
  }: {
    user: string;
  }) => Promise<any>;
  accountInformation: ({
    user
  }: {
    user: string;
  }) => Promise<any>;
  accountCollateralValueUsd: ({
    user
  }: {
    user: string;
  }) => Promise<any>;
  calculateHealthFactor: ({
    collateralValueUsd,
    totalDscMinted
  }: {
    collateralValueUsd: Decimal;
    totalDscMinted: Uint128;
  }) => Promise<any>;
  getUsdValue: ({
    amount,
    token
  }: {
    amount: Uint128;
    token: string;
  }) => Promise<any>;
  getTokenAmountFromUsd: ({
    token,
    usdAmount
  }: {
    token: string;
    usdAmount: Decimal;
  }) => Promise<any>;
  getCollateralTokenPriceFeed: ({
    collateralAsset
  }: {
    collateralAsset: string;
  }) => Promise<any>;
  getCollateralBalanceOfUser: ({
    token,
    user
  }: {
    token: string;
    user: string;
  }) => Promise<any>;
}
export class DscEngineQueryContract extends Contract implements DscEngineReadOnlyInterface {
  constructor(contractName: string, instantiateTag?: string) {
    super(contractName, instantiateTag);
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
  config = async (): Promise<any> => {
    return this.queryMsg({
      config: {}
    });
  };
  collateralBalanceOfUser = async ({
    collateralAsset,
    user
  }: {
    collateralAsset: string;
    user: string;
  }): Promise<any> => {
    return this.queryMsg({
      collateral_balance_of_user: {
        collateral_asset: collateralAsset,
        user
      }
    });
  };
  userHealthFactor = async ({
    user
  }: {
    user: string;
  }): Promise<any> => {
    return this.queryMsg({
      user_health_factor: {
        user
      }
    });
  };
  accountInformation = async ({
    user
  }: {
    user: string;
  }): Promise<any> => {
    return this.queryMsg({
      account_information: {
        user
      }
    });
  };
  accountCollateralValueUsd = async ({
    user
  }: {
    user: string;
  }): Promise<any> => {
    return this.queryMsg({
      account_collateral_value_usd: {
        user
      }
    });
  };
  calculateHealthFactor = async ({
    collateralValueUsd,
    totalDscMinted
  }: {
    collateralValueUsd: Decimal;
    totalDscMinted: Uint128;
  }): Promise<any> => {
    return this.queryMsg({
      calculate_health_factor: {
        collateral_value_usd: collateralValueUsd,
        total_dsc_minted: totalDscMinted
      }
    });
  };
  getUsdValue = async ({
    amount,
    token
  }: {
    amount: Uint128;
    token: string;
  }): Promise<any> => {
    return this.queryMsg({
      get_usd_value: {
        amount,
        token
      }
    });
  };
  getTokenAmountFromUsd = async ({
    token,
    usdAmount
  }: {
    token: string;
    usdAmount: Decimal;
  }): Promise<any> => {
    return this.queryMsg({
      get_token_amount_from_usd: {
        token,
        usd_amount: usdAmount
      }
    });
  };
  getCollateralTokenPriceFeed = async ({
    collateralAsset
  }: {
    collateralAsset: string;
  }): Promise<any> => {
    return this.queryMsg({
      get_collateral_token_price_feed: {
        collateral_asset: collateralAsset
      }
    });
  };
  getCollateralBalanceOfUser = async ({
    token,
    user
  }: {
    token: string;
    user: string;
  }): Promise<any> => {
    return this.queryMsg({
      get_collateral_balance_of_user: {
        token,
        user
      }
    });
  };
}
export interface DscEngineInterface extends DscEngineReadOnlyInterface {
  depositCollateralAndMintDsc: ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    amountCollateral,
    amountDscToMint,
    collateralAsset
  }: {
    amountCollateral: Uint128;
    amountDscToMint: Uint128;
    collateralAsset: AssetInfoBase_for_Addr;
  }) => Promise<any>;
  redeemCollateralForDsc: ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    amountCollateral,
    amountDscToBurn,
    collateralAsset
  }: {
    amountCollateral: Uint128;
    amountDscToBurn: Uint128;
    collateralAsset: AssetInfoBase_for_Addr;
  }) => Promise<any>;
  redeemCollateral: ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    amountCollateral,
    collateralAsset
  }: {
    amountCollateral: Uint128;
    collateralAsset: AssetInfoBase_for_Addr;
  }) => Promise<any>;
  burnDsc: ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    amountDscToBurn
  }: {
    amountDscToBurn: Uint128;
  }) => Promise<any>;
  liquidate: ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    collateralAsset,
    debtToCover,
    user
  }: {
    collateralAsset: AssetInfoBase_for_Addr;
    debtToCover: Decimal;
    user: string;
  }) => Promise<any>;
}
export class DscEngineContract extends DscEngineQueryContract implements DscEngineInterface {
  constructor(instantiateTag?: string) {
    super("dsc_engine", instantiateTag);
    this.depositCollateralAndMintDsc = this.depositCollateralAndMintDsc.bind(this);
    this.redeemCollateralForDsc = this.redeemCollateralForDsc.bind(this);
    this.redeemCollateral = this.redeemCollateral.bind(this);
    this.burnDsc = this.burnDsc.bind(this);
    this.liquidate = this.liquidate.bind(this);
  }
  depositCollateralAndMintDsc = async ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    amountCollateral,
    amountDscToMint,
    collateralAsset
  }: {
    amountCollateral: Uint128;
    amountDscToMint: Uint128;
    collateralAsset: AssetInfoBase_for_Addr;
  }): Promise<any> => {
    return await this.executeMsg({
      deposit_collateral_and_mint_dsc: {
        amount_collateral: amountCollateral,
        amount_dsc_to_mint: amountDscToMint,
        collateral_asset: collateralAsset
      }
    }, account, customFees, memo, transferAmount);
  };
  redeemCollateralForDsc = async ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    amountCollateral,
    amountDscToBurn,
    collateralAsset
  }: {
    amountCollateral: Uint128;
    amountDscToBurn: Uint128;
    collateralAsset: AssetInfoBase_for_Addr;
  }): Promise<any> => {
    return await this.executeMsg({
      redeem_collateral_for_dsc: {
        amount_collateral: amountCollateral,
        amount_dsc_to_burn: amountDscToBurn,
        collateral_asset: collateralAsset
      }
    }, account, customFees, memo, transferAmount);
  };
  redeemCollateral = async ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    amountCollateral,
    collateralAsset
  }: {
    amountCollateral: Uint128;
    collateralAsset: AssetInfoBase_for_Addr;
  }): Promise<any> => {
    return await this.executeMsg({
      redeem_collateral: {
        amount_collateral: amountCollateral,
        collateral_asset: collateralAsset
      }
    }, account, customFees, memo, transferAmount);
  };
  burnDsc = async ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    amountDscToBurn
  }: {
    amountDscToBurn: Uint128;
  }): Promise<any> => {
    return await this.executeMsg({
      burn_dsc: {
        amount_dsc_to_burn: amountDscToBurn
      }
    }, account, customFees, memo, transferAmount);
  };
  liquidate = async ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }, {
    collateralAsset,
    debtToCover,
    user
  }: {
    collateralAsset: AssetInfoBase_for_Addr;
    debtToCover: Decimal;
    user: string;
  }): Promise<any> => {
    return await this.executeMsg({
      liquidate: {
        collateral_asset: collateralAsset,
        debt_to_cover: debtToCover,
        user
      }
    }, account, customFees, memo, transferAmount);
  };
}