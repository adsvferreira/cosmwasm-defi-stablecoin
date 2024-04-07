import { Contract, wasmKitTypes, Coin } from "@kubiklabs/wasmkit";
export type ExecuteMsg = {
    increment: {
        [k: string]: unknown;
    };
} | {
    reset: {
        count: number;
        [k: string]: unknown;
    };
};
export interface InstantiateMsg {
    count: number;
    [k: string]: unknown;
}
export type QueryMsg = {
    get_count: {
        [k: string]: unknown;
    };
};
export interface CounterReadOnlyInterface {
    getCount: () => Promise<any>;
}
export declare class CounterQueryContract extends Contract implements CounterReadOnlyInterface {
    constructor(contractName: string, instantiateTag?: string);
    getCount: () => Promise<any>;
}
export interface CounterInterface extends CounterReadOnlyInterface {
    increment: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }) => Promise<any>;
    reset: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee;
        memo?: string;
        transferAmount?: readonly Coin[];
    }, { count }: {
        count: number;
    }) => Promise<any>;
}
export declare class CounterContract extends CounterQueryContract implements CounterInterface {
    constructor(instantiateTag?: string);
    increment: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }) => Promise<any>;
    reset: ({ account, customFees, memo, transferAmount }: {
        account: wasmKitTypes.UserAccount;
        customFees?: wasmKitTypes.TxnStdFee | undefined;
        memo?: string | undefined;
        transferAmount?: readonly wasmKitTypes.Coin[] | undefined;
    }, { count }: {
        count: number;
    }) => Promise<any>;
}
