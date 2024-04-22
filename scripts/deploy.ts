import { getAccountByName } from "@kubiklabs/wasmkit";

import { DecentralizedStablecoinContract } from "../artifacts/typescript_schema/DecentralizedStablecoinContract";
import { DscEngineContract } from "../artifacts/typescript_schema/DscEngineContract";
import {  OracleContract} from "../artifacts/typescript_schema/OracleContract";
import Decimal from 'decimal.js';


export default async function run () {
  const runTs = String(new Date());

  const contract_owner = await getAccountByName("account_0");
  const contract_owner_address = contract_owner.account.address;

  const pyth_oracle_addr = "neutron1m2emc93m9gpwgsrsf2vylv9xvgqh654630v7dfrhrkmr5slly53spg85wv"
  const ntrn_usd_price_feed_id = "a8e6517966a52cb1df864b2764f3629fde3f21d2b640b5c572fcd654cbccd65e"
  const native_ntrn_denom = "untrn"
  const liq_thresold = "50"; // 200% collaterized
  const liq_bonus = "10" // 10% 
  const min_health_factor = "1.0"

  // 1 - Deploy DSC CW20

  const stable_cw20_contract = new DecentralizedStablecoinContract();
  await stable_cw20_contract.setupClient();

  const dsc_deploy_response = await stable_cw20_contract.deploy(
    contract_owner,
  );
  console.log("DSC DEPLOY RES:");
  console.log(dsc_deploy_response);
  console.log();

  const dsc_info = await stable_cw20_contract.instantiate(
    {
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

    },
    `deploy test ${runTs}`,
    contract_owner,
    undefined, // tokens to tranfer
    undefined, // customFees, // custom fess here
    contract_owner.account.address, // contractAdmin
  );

  console.log("DSC INSTANTIATE RES:");
  console.log(dsc_info);
  console.log();

  const dsc_addr = dsc_info.contractAddress;
  
  const mint_res = await stable_cw20_contract.mint(
    {account: contract_owner}, 
    {recipient: contract_owner_address, amount: "1000000"}
  )

  console.log(mint_res)

  const balance_query_res = await stable_cw20_contract.balance(
    {address: contract_owner_address}
  )

  console.log("CURRENT BALANCE")
  console.log(balance_query_res)

  // 2 - Deploy Oracle

  const oracle_contract = new OracleContract();
  await oracle_contract.setupClient();

  const oracle_deploy_response = await oracle_contract.deploy(
    contract_owner,
  );
  console.log("ORACLE DEPLOY RES:");
  console.log(oracle_deploy_response);
  console.log();

  const oracle_info = await oracle_contract.instantiate(
    {},
    `deploy test ${runTs}`,
    contract_owner,
    undefined, // tokens to tranfer
    undefined, // customFees, // custom fess here
    contract_owner.account.address, // contractAdmin
  );

  console.log("ORACLE INSTANTIATE RES:");
  console.log(oracle_info);
  console.log();

  const oracle_addr = oracle_info.contractAddress;
  console.log("ORACLE ADDR: ", oracle_addr)


  // 3 - Deploy DSCE

  const dsce_contract = new DscEngineContract();
  await dsce_contract.setupClient();

  const dsce_deploy_response = await dsce_contract.deploy(
    contract_owner,
  );
  console.log("DSC ENGINE DEPLOY RES:");
  console.log(dsce_deploy_response);
  console.log();

  const dsce_info = await dsce_contract.instantiate(
    {
      "owner": contract_owner.account.address,
      "assets": [
        {
          native : native_ntrn_denom,
        }
      ],
      "oracle_address": oracle_addr,
      "pyth_oracle_address": pyth_oracle_addr,
      "price_feed_ids": [ntrn_usd_price_feed_id],
      "dsc_address": dsc_addr,
      "liquidation_threshold": liq_thresold,
      "liquidation_bonus": liq_bonus,
      "min_health_factor": min_health_factor
    },
    `deploy test ${runTs}`,
    contract_owner,
    undefined, // tokens to tranfer
    undefined, // customFees, // custom fess here
    contract_owner.account.address, // contractAdmin
  );

  console.log("DSC ENGINE INSTANTIATE RES:")
  console.log(dsc_info);
  console.log();

  const dsce_address = dsce_info.contractAddress;
  console.log("DSCE ADDR:")
  console.log(dsce_address)
  console.log()

  // 4 - Update DSC minter to DSCE

  const update_minter_res = await stable_cw20_contract.updateMinter(
    {account: contract_owner}, 
    {new_minter: dsce_address}
  )

  console.log("UPDATE MINTER RES:");
  console.log(update_minter_res);
  console.log();

  // ------ Test the code above on localnet
  // ------ Deploy and test the code below on mainnet
  // 5 - execute deposit_collateral_and_mint_dsc

}
