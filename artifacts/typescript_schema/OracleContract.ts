import { Contract, wasmKitTypes, Coin } from "@kubiklabs/wasmkit";
export interface InstantiateMsg {}
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
  fetchPrice: ({
    priceFeedId,
    pythContractAddr
  }: {
    priceFeedId: Identifier;
    pythContractAddr: string;
  }) => Promise<any>;
  fetchValidTimePeriod: ({
    pythContractAddr
  }: {
    pythContractAddr: string;
  }) => Promise<any>;
}
export class OracleQueryContract extends Contract implements OracleReadOnlyInterface {
  constructor(contractName: string, instantiateTag?: string) {
    super(contractName, instantiateTag);
    this.fetchPrice = this.fetchPrice.bind(this);
    this.fetchValidTimePeriod = this.fetchValidTimePeriod.bind(this);
  }
  fetchPrice = async ({
    priceFeedId,
    pythContractAddr
  }: {
    priceFeedId: Identifier;
    pythContractAddr: string;
  }): Promise<any> => {
    return this.queryMsg({
      fetch_price: {
        price_feed_id: priceFeedId,
        pyth_contract_addr: pythContractAddr
      }
    });
  };
  fetchValidTimePeriod = async ({
    pythContractAddr
  }: {
    pythContractAddr: string;
  }): Promise<any> => {
    return this.queryMsg({
      fetch_valid_time_period: {
        pyth_contract_addr: pythContractAddr
      }
    });
  };
}