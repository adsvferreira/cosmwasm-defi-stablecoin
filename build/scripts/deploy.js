"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const wasmkit_1 = require("@kubiklabs/wasmkit");
const DscContract_1 = require("../artifacts/typescript_schema/DscContract");
const DscEngineContract_1 = require("../artifacts/typescript_schema/DscEngineContract");
const OracleContract_1 = require("../artifacts/typescript_schema/OracleContract");
async function run() {
    const runTs = String(new Date());
    const contract_owner = await (0, wasmkit_1.getAccountByName)("account_0");
    const contract_owner_address = contract_owner.account.address;
    const pyth_oracle_addr = "neutron1m2emc93m9gpwgsrsf2vylv9xvgqh654630v7dfrhrkmr5slly53spg85wv";
    const ntrn_usd_price_feed_id = "a8e6517966a52cb1df864b2764f3629fde3f21d2b640b5c572fcd654cbccd65e";
    const native_ntrn_denom = "untrn";
    const liq_thresold = "50"; // 200% collaterized
    const liq_bonus = "10"; // 10% 
    const min_health_factor = "1.0";
    const stable_cw20_contract = new DscContract_1.DscContract();
    await stable_cw20_contract.setupClient();
    const oracle_contract = new OracleContract_1.OracleContract();
    await oracle_contract.setupClient();
    const dsce_contract = new DscEngineContract_1.DscEngineContract();
    await dsce_contract.setupClient();
    // 1 - Deploy DSC CW20
    const dsc_deploy_response = await stable_cw20_contract.deploy(contract_owner);
    console.log("DSC DEPLOY RES:");
    console.log(dsc_deploy_response);
    console.log();
    const dsc_info = await stable_cw20_contract.instantiate({
        "name": "Decentralized Stablecoin",
        "symbol": "DSC",
        "decimals": 6,
        "initial_balances": [
            {
                address: contract_owner_address,
                amount: "10000000"
            }
        ],
        "mint": {
            "minter": contract_owner_address,
            "cap": "1000000000000",
        }
    }, `deploy dsc ${runTs}`, contract_owner, undefined, // tokens to tranfer
    undefined, // customFees, // custom fess here
    contract_owner.account.address);
    console.log("DSC INSTANTIATE RES:");
    console.log(dsc_info);
    console.log();
    const dsc_addr = dsc_info.contractAddress;
    // 2 - Deploy Oracle
    const oracle_deploy_response = await oracle_contract.deploy(contract_owner);
    console.log("ORACLE DEPLOY RES:");
    console.log(oracle_deploy_response);
    console.log();
    const oracle_info = await oracle_contract.instantiate({}, `deploy oracle ${runTs}`, contract_owner, undefined, // tokens to tranfer
    undefined, // customFees, // custom fees here
    contract_owner.account.address);
    console.log("ORACLE INSTANTIATE RES:");
    console.log(oracle_info);
    console.log();
    const oracle_addr = oracle_info.contractAddress;
    console.log("ORACLE ADDR: ", oracle_addr);
    // 3 - Deploy DSCE
    const dsce_deploy_response = await dsce_contract.deploy(contract_owner);
    console.log("DSC ENGINE DEPLOY RES:");
    console.log(dsce_deploy_response);
    console.log();
    const dsce_info = await dsce_contract.instantiate({
        "owner": contract_owner.account.address,
        "assets": [
            {
                native: native_ntrn_denom,
            }
        ],
        "oracle_address": oracle_addr,
        "pyth_oracle_address": pyth_oracle_addr,
        "price_feed_ids": [ntrn_usd_price_feed_id],
        "dsc_address": dsc_addr,
        "liquidation_threshold": liq_thresold,
        "liquidation_bonus": liq_bonus,
        "min_health_factor": min_health_factor
    }, `deploy dsce ${runTs}`, contract_owner, undefined, // tokens to tranfer
    undefined, // customFees, // custom fess here
    contract_owner.account.address);
    console.log("DSC ENGINE INSTANTIATE RES:");
    console.log(dsc_info);
    console.log();
    const dsce_address = dsce_info.contractAddress;
    console.log("DSCE ADDR:");
    console.log(dsce_address);
    console.log();
    // 4 - Update DSC minter to DSCE
    const update_minter_res = await stable_cw20_contract.updateMinter({ account: contract_owner }, { newMinter: dsce_address });
    console.log("UPDATE MINTER RES:");
    console.log(update_minter_res);
    console.log();
    // 5 - increase allowance of cw20 from user to dsce contract
    const increase_allowance_res = await stable_cw20_contract.increaseAllowance({ account: contract_owner }, { spender: dsce_address, amount: "3000000", expires: null });
    console.log("INCREASE ALLOWANCE RES:");
    console.log(increase_allowance_res);
    console.log();
    // 6 - execute deposit_collateral_and_mint_dsc
    const deposit_collateral_and_mint_dsc_res = await dsce_contract.depositCollateralAndMintDsc({ account: contract_owner, transferAmount: [{ denom: native_ntrn_denom, amount: "2000000" }] }, {
        collateralAsset: { native: native_ntrn_denom },
        amountCollateral: "2000000",
        amountDscToMint: "1000000"
    });
    console.log("DEPOSIT AND MINT RES:");
    console.log(deposit_collateral_and_mint_dsc_res);
    console.log();
    const dsce_config = await dsce_contract.config();
    console.log("CONFIG");
    console.log(dsce_config);
}
exports.default = run;
//# sourceMappingURL=deploy.js.map