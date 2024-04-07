import { Contract, wasmKitTypes, Coin } from "@kubiklabs/wasmkit";
export type Cw20ExecuteMsg = {
    transfer: {
        amount: Uint128;
        recipient: string;
    };
} | {
    burn: {
        amount: Uint128;
    };
} | {
    send: {
        amount: Uint128;
        contract: string;
        msg: Binary;
    };
} | {
    increase_allowance: {
        amount: Uint128;
        expires?: Expiration | null;
        spender: string;
    };
} | {
    decrease_allowance: {
        amount: Uint128;
        expires?: Expiration | null;
        spender: string;
    };
} | {
    transfer_from: {
        amount: Uint128;
        owner: string;
        recipient: string;
    };
} | {
    send_from: {
        amount: Uint128;
        contract: string;
        msg: Binary;
        owner: string;
    };
} | {
    burn_from: {
        amount: Uint128;
        owner: string;
    };
} | {
    mint: {
        amount: Uint128;
        recipient: string;
    };
} | {
    update_minter: {
        new_minter?: string | null;
    };
} | {
    update_marketing: {
        description?: string | null;
        marketing?: string | null;
        project?: string | null;
    };
} | {
    upload_logo: Logo;
};
export type Uint128 = string;
export type Binary = string;
export type Expiration = {
    at_height: number;
} | {
    at_time: Timestamp;
} | {
    never: {};
};
export type Timestamp = Uint64;
export type Uint64 = string;
export type Logo = {
    url: string;
} | {
    embedded: EmbeddedLogo;
};
export type EmbeddedLogo = {
    svg: Binary;
} | {
    png: Binary;
};
export interface InstantiateMsg {
    decimals: number;
    initial_balances: Cw20Coin[];
    marketing?: InstantiateMarketingInfo | null;
    mint?: MinterResponse | null;
    name: string;
    symbol: string;
}
export interface Cw20Coin {
    address: string;
    amount: Uint128;
}
export interface InstantiateMarketingInfo {
    description?: string | null;
    logo?: Logo | null;
    marketing?: string | null;
    project?: string | null;
}
export interface MinterResponse {
    cap?: Uint128 | null;
    minter: string;
}
export type QueryMsg = {
    balance: {
        address: string;
    };
} | {
    token_info: {};
} | {
    minter: {};
} | {
    allowance: {
        owner: string;
        spender: string;
    };
} | {
    all_allowances: {
        limit?: number | null;
        owner: string;
        start_after?: string | null;
    };
} | {
    all_spender_allowances: {
        limit?: number | null;
        spender: string;
        start_after?: string | null;
    };
} | {
    all_accounts: {
        limit?: number | null;
        start_after?: string | null;
    };
} | {
    marketing_info: {};
} | {
    download_logo: {};
};
export interface DecentralizedStablecoinReadOnlyInterface {
    balance: ({ address }: {
        address: string;
    }) => Promise<any>;
    tokenInfo: () => Promise<any>;
    minter: () => Promise<any>;
    allowance: ({ owner, spender }: {
        owner: string;
        spender: string;
    }) => Promise<any>;
    allAllowances: ({ limit, owner, startAfter }: {
        limit: number | null;
        owner: string;
        startAfter: string | null;
    }) => Promise<any>;
    allSpenderAllowances: ({ limit, spender, startAfter }: {
        limit: number | null;
        spender: string;
        startAfter: string | null;
    }) => Promise<any>;
    allAccounts: ({ limit, startAfter }: {
        limit: number | null;
        startAfter: string | null;
    }) => Promise<any>;
    marketingInfo: () => Promise<any>;
    downloadLogo: () => Promise<any>;
}
export declare class DecentralizedStablecoinQueryContract extends Contract implements DecentralizedStablecoinReadOnlyInterface {
    constructor(contractName: string, instantiateTag?: string);
    balance: ({ address }: {
        address: string;
    }) => Promise<any>;
    tokenInfo: () => Promise<any>;
    minter: () => Promise<any>;
    allowance: ({ owner, spender }: {
        owner: string;
        spender: string;
    }) => Promise<any>;
    allAllowances: ({ limit, owner, startAfter }: {
        limit: number | null;
        owner: string;
        startAfter: string | null;
    }) => Promise<any>;
    allSpenderAllowances: ({ limit, spender, startAfter }: {
        limit: number | null;
        spender: string;
        startAfter: string | null;
    }) => Promise<any>;
    allAccounts: ({ limit, startAfter }: {
        limit: number | null;
        startAfter: string | null;
    }) => Promise<any>;
    marketingInfo: () => Promise<any>;
    downloadLogo: () => Promise<any>;
}
export interface DecentralizedStablecoinInterface extends DecentralizedStablecoinReadOnlyInterface {
    transfer: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amount, recipient }: {
        amount: Uint128;
        recipient: string;
    }) => Promise<any>;
    burn: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amount }: {
        amount: Uint128;
    }) => Promise<any>;
    send: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amount, contract, msg }: {
        amount: Uint128;
        contract: string;
        msg: Binary;
    }) => Promise<any>;
    increaseAllowance: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amount, expires, spender }: {
        amount: Uint128;
        expires: Expiration | null;
        spender: string;
    }) => Promise<any>;
    decreaseAllowance: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amount, expires, spender }: {
        amount: Uint128;
        expires: Expiration | null;
        spender: string;
    }) => Promise<any>;
    transferFrom: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amount, owner, recipient }: {
        amount: Uint128;
        owner: string;
        recipient: string;
    }) => Promise<any>;
    sendFrom: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amount, contract, msg, owner }: {
        amount: Uint128;
        contract: string;
        msg: Binary;
        owner: string;
    }) => Promise<any>;
    burnFrom: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amount, owner }: {
        amount: Uint128;
        owner: string;
    }) => Promise<any>;
    mint: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { amount, recipient }: {
        amount: Uint128;
        recipient: string;
    }) => Promise<any>;
    updateMinter: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { newMinter }: {
        newMinter: string | null;
    }) => Promise<any>;
    updateMarketing: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { description, marketing, project }: {
        description: string | null;
        marketing: string | null;
        project: string | null;
    }) => Promise<any>;
    uploadLogo: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }) => Promise<any>;
}
export declare class DecentralizedStablecoinContract extends DecentralizedStablecoinQueryContract implements DecentralizedStablecoinInterface {
    constructor(instantiateTag?: string);
    transfer: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amount, recipient }: {
        amount: Uint128;
        recipient: string;
    }) => Promise<any>;
    burn: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amount }: {
        amount: Uint128;
    }) => Promise<any>;
    send: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amount, contract, msg }: {
        amount: Uint128;
        contract: string;
        msg: Binary;
    }) => Promise<any>;
    increaseAllowance: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amount, expires, spender }: {
        amount: Uint128;
        expires: Expiration | null;
        spender: string;
    }) => Promise<any>;
    decreaseAllowance: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amount, expires, spender }: {
        amount: Uint128;
        expires: Expiration | null;
        spender: string;
    }) => Promise<any>;
    transferFrom: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amount, owner, recipient }: {
        amount: Uint128;
        owner: string;
        recipient: string;
    }) => Promise<any>;
    sendFrom: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amount, contract, msg, owner }: {
        amount: Uint128;
        contract: string;
        msg: Binary;
        owner: string;
    }) => Promise<any>;
    burnFrom: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amount, owner }: {
        amount: Uint128;
        owner: string;
    }) => Promise<any>;
    mint: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { amount, recipient }: {
        amount: Uint128;
        recipient: string;
    }) => Promise<any>;
    updateMinter: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { newMinter }: {
        newMinter: string | null;
    }) => Promise<any>;
    updateMarketing: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { description, marketing, project }: {
        description: string | null;
        marketing: string | null;
        project: string | null;
    }) => Promise<any>;
    uploadLogo: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }) => Promise<any>;
}
