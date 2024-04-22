import { Contract, wasmKitTypes, Coin } from "@kubiklabs/wasmkit";
export type ExecuteMsg = {
  update_mock_price: {
    price: number;
  };
};
export interface MockPythInterface {
  updateMockPrice: ({
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
    price
  }: {
    price: number;
  }) => Promise<any>;
}
export class MockPythContract implements MockPythInterface {
  constructor(instantiateTag?: string) {
    this.updateMockPrice = this.updateMockPrice.bind(this);
  }
  updateMockPrice = async ({
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
    price
  }: {
    price: number;
  }): Promise<any> => {
    return await this.executeMsg({
      update_mock_price: {
        price
      }
    }, account, customFees, memo, transferAmount);
  };
}