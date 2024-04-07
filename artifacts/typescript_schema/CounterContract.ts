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
export class CounterQueryContract extends Contract implements CounterReadOnlyInterface {
  constructor(contractName: string, instantiateTag?: string) {
    super(contractName, instantiateTag);
    this.getCount = this.getCount.bind(this);
  }
  getCount = async (): Promise<any> => {
    return this.queryMsg({
      get_count: {}
    });
  };
}
export interface CounterInterface extends CounterReadOnlyInterface {
  increment: ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }) => Promise<any>;
  reset: ({
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
    count
  }: {
    count: number;
  }) => Promise<any>;
}
export class CounterContract extends CounterQueryContract implements CounterInterface {
  constructor(instantiateTag?: string) {
    super("counter", instantiateTag);
    this.increment = this.increment.bind(this);
    this.reset = this.reset.bind(this);
  }
  increment = async ({
    account,
    customFees,
    memo,
    transferAmount
  }: {
    account: wasmKitTypes.UserAccount;
    customFees?: wasmKitTypes.TxnStdFee;
    memo?: string;
    transferAmount?: readonly Coin[];
  }): Promise<any> => {
    return await this.executeMsg({
      increment: {}
    }, account, customFees, memo, transferAmount);
  };
  reset = async ({
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
    count
  }: {
    count: number;
  }): Promise<any> => {
    return await this.executeMsg({
      reset: {
        count
      }
    }, account, customFees, memo, transferAmount);
  };
}