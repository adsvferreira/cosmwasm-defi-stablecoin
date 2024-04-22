"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.OracleContract = exports.OracleQueryContract = void 0;
const wasmkit_1 = require("@kubiklabs/wasmkit");
class OracleQueryContract extends wasmkit_1.Contract {
    constructor(contractName, instantiateTag) {
        super(contractName, instantiateTag);
        this.fetchPrice = async ({ priceFeedId, pythContractAddr }) => {
            return this.queryMsg({
                fetch_price: {
                    price_feed_id: priceFeedId,
                    pyth_contract_addr: pythContractAddr
                }
            });
        };
        this.fetchValidTimePeriod = async ({ pythContractAddr }) => {
            return this.queryMsg({
                fetch_valid_time_period: {
                    pyth_contract_addr: pythContractAddr
                }
            });
        };
        this.fetchPrice = this.fetchPrice.bind(this);
        this.fetchValidTimePeriod = this.fetchValidTimePeriod.bind(this);
    }
}
exports.OracleQueryContract = OracleQueryContract;
class OracleContract extends OracleQueryContract {
    constructor(instantiateTag) {
        super("oracle", instantiateTag);
    }
    ;
}
exports.OracleContract = OracleContract;
//# sourceMappingURL=OracleContract.js.map