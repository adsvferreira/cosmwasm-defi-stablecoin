import { Contract } from "@kubiklabs/wasmkit";
export interface InstantiateMsg {
}
export type QueryMsg = {
    fetch_price: {
        price_feed_id: Identifier;
        pyth_contract_addr: string;
    };
} | {
    fetch_valid_time_period: {
        pyth_contract_addr: string;
    };
};
export type Identifier = string;
export interface OracleReadOnlyInterface {
    fetchPrice: ({ priceFeedId, pythContractAddr }: {
        priceFeedId: Identifier;
        pythContractAddr: string;
    }) => Promise<any>;
    fetchValidTimePeriod: ({ pythContractAddr }: {
        pythContractAddr: string;
    }) => Promise<any>;
}
export declare class OracleQueryContract extends Contract implements OracleReadOnlyInterface {
    constructor(contractName: string, instantiateTag?: string);
    fetchPrice: ({ priceFeedId, pythContractAddr }: {
        priceFeedId: Identifier;
        pythContractAddr: string;
    }) => Promise<any>;
    fetchValidTimePeriod: ({ pythContractAddr }: {
        pythContractAddr: string;
    }) => Promise<any>;
}
export declare class OracleContract extends OracleQueryContract implements OracleReadOnlyInterface {
    constructor(instantiateTag?: string);
}
