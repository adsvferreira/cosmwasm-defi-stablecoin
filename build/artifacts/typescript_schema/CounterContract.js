"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CounterContract = exports.CounterQueryContract = void 0;
const wasmkit_1 = require("@kubiklabs/wasmkit");
class CounterQueryContract extends wasmkit_1.Contract {
    constructor(contractName, instantiateTag) {
        super(contractName, instantiateTag);
        this.getCount = async () => {
            return this.queryMsg({
                get_count: {}
            });
        };
        this.getCount = this.getCount.bind(this);
    }
}
exports.CounterQueryContract = CounterQueryContract;
class CounterContract extends CounterQueryContract {
    constructor(instantiateTag) {
        super("counter", instantiateTag);
        this.increment = async ({ account, customFees, memo, transferAmount }) => {
            return await this.executeMsg({
                increment: {}
            }, account, customFees, memo, transferAmount);
        };
        this.reset = async ({ account, customFees, memo, transferAmount }, { count }) => {
            return await this.executeMsg({
                reset: {
                    count
                }
            }, account, customFees, memo, transferAmount);
        };
        this.increment = this.increment.bind(this);
        this.reset = this.reset.bind(this);
    }
}
exports.CounterContract = CounterContract;
//# sourceMappingURL=CounterContract.js.map