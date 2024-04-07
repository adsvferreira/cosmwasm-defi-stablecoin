"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const wasmkit_1 = require("@kubiklabs/wasmkit");
const DecentralizedStablecoinContract_1 = require("../artifacts/typescript_schema/DecentralizedStablecoinContract");
async function run() {
    const runTs = String(new Date());
    const contract_owner = await (0, wasmkit_1.getAccountByName)("account_0");
    const contract_owner_address = contract_owner.account.address;
    const stable_cw20_contract = new DecentralizedStablecoinContract_1.DecentralizedStablecoinContract();
    await stable_cw20_contract.setupClient();
    const deploy_response = await stable_cw20_contract.deploy(contract_owner);
    console.log(deploy_response);
    const contract_info = await stable_cw20_contract.instantiate({
        "name": "Stop Women From Voting Token",
        "symbol": "SWFVT",
        "decimals": 6,
        "initial_balances": [
            {
                address: contract_owner_address,
                amount: "100000000000"
            }
        ],
        "mint": {
            "minter": contract_owner_address,
            "cap": "1000000000000",
        }
    }, `deploy test ${runTs}`, contract_owner, undefined, // tokens to tranfer
    undefined, // customFees, // custom fess here
    contract_owner.account.address);
    console.log(contract_info);
    // const mint_res = await stable_cw20_contract.mint(
    //   {account: contract_owner}, 
    //   {recipient: contract_owner_address, amount: "1000000"}
    // )
    // console.log(mint_res)
    const balance_query_res = await stable_cw20_contract.balance({ address: contract_owner_address });
    console.log("CURRENT BALANCE");
    console.log(balance_query_res);
    // const inc_response = await counter_contract.increment({account: contract_owner});
    // console.log(inc_response);
    // const response = await counter_contract.getCount();
    // console.log(response);
    // const ex_response = await counter_contract.increment(
    //   {
    //     account: contract_owner,
    //   }
    // );
    // console.log(ex_response);
}
exports.default = run;
//# sourceMappingURL=sample-script.js.map