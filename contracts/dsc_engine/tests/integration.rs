#[cfg(test)]
use cosmwasm_std::{coins, Addr, Coin, Decimal, Empty, Uint128};
use cw20::Cw20ExecuteMsg;
use cw20::{BalanceResponse, Cw20Coin, MinterResponse, TokenInfoResponse};
use cw20_base::contract::{
    execute as cw20_execute, instantiate as cw20_instantiate, query as cw20_query,
};
use cw20_base::msg::{InstantiateMsg as Cw20InstantiateMsg, QueryMsg as Cw20QueryMsg};
use cw_asset::AssetInfo;
use cw_multi_test::{App, ContractWrapper, Executor};
use dsc::contract::{execute as dsc_execute, instantiate as dsc_instantiate, query as dsc_query};
use dsc_engine::contract::{execute, instantiate};
use dsc_engine::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use dsc_engine::queries::query;
use mock_pyth::contract::{
    execute as mock_pyth_execute, instantiate as mock_pyth_instantiate, query as mock_pyth_query,
};
use mock_pyth::msg::ExecuteMsg as MockPythExecuteMsg;
use oracle::contract::{
    execute as oracle_execute, instantiate as oracle_instantiate, query as oracle_query,
};
use oracle::msg::InstantiateMsg as OracleInstantiateMsg;
use oracle::msg::{FetchPriceResponse, QueryMsg as OracleQueryMsg};
use pyth_sdk_cw::PriceIdentifier;
use std::collections::HashMap;

const OWNER: &str = "neutron17ykn47jnxnn83ceh95grafvtjx7xzsstw0jq9d";
const LIQUIDATOR: &str = "neutron1utl04swwlt9xsr9c2vdx2nkea4plp66sx3s8pg";
const LIQ_THRESHOLD: Uint128 = Uint128::new(50);
const LIQ_BONUS: Uint128 = Uint128::new(10);
const MIN_HEALTH_FACTOR: Decimal = Decimal::one();
const NATIVE_COLLATERAL_DENOM: &str = "native";
const INITIAL_OWNER_NATIVE_BALANCE: u128 = 15_000_000;
const CW20_COLLATERAL_DENOM: &str = "address";
const CW20_AMOUNT_MINTED_TO_OWNER: Uint128 = Uint128::new(1_000_000_000_000);
const ORACLE_ADDRESS: &str = "oracle_addr";
const PYTH_ORACLE_ADDRESS: &str = "pyth_oracle_addr";
const PRICE_FEED_ID_1: &str = "63f341689d98a12ef60a5cff1d7f85c70a9e17bf1575f0e7c0b2512d48b1c8b3";
const PRICE_FEED_ID_2: &str = "2b9ab1e972a281585084148ba1389800799bd4be63b957507db1349314e47445";
const DSC_ADDR: &str = "dsc_addr";
const AMOUNT_COLLATERAL_OK: Uint128 = Uint128::new(2_000_000); // 2_000_000/1_000_000 * 680_000/100_000 = 13.6 usd at mock oracle price
const AMOUNT_DSC_TO_MINT_OK: Uint128 = Uint128::new(1_000_000); // health factor = 6.8
const DEBT_TO_COVER: Uint128 = Uint128::new(900_000);
const LIQUIDATION_PRICE: i64 = 97_000;
const FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR: Uint128 = Uint128::new(4_000_000); // Same as initial
const FINAL_COLLATERAL_BALANCE_OF_LIQUIDATED: Uint128 = Uint128::new(979_382); // 2_000_000/1_000_000 - (900_000/1_000_000 * 100_000/97_000) * 1.1
const FINAL_DSC_BALANCE_OF_LIQUIDATOR: Uint128 = Uint128::new(100_000); // 1_000_000 - 900_000
const FINAL_DSC_BALANCE_OF_LIQUIDATED: Uint128 = Uint128::new(1_000_000); // Same as initial
const FINAL_DSC_SUPPLY: Uint128 = Uint128::new(1_100_000); // 2_000_000 - 900_000
const FINAL_BALANCE_OF_LIQUIDATOR: Uint128 = Uint128::new(1_020_618); // (900_000/1_000_000 * 100_000/97_000) * 1.1
const FINAL_NATIVE_BALANCE_OF_LIQUIDATED: Uint128 = Uint128::new(9_000_000); // 15_000_000 - 4_000_000 - 2_000_000
const FINAL_CW20_BALANCE_OF_LIQUIDATED: Uint128 = Uint128::new(13_000_000); // 15_000_000 - 2_000_000

fn get_default_instantiate_msg(
    cw20_address: Option<&str>,
    dsc_address: Option<&str>,
    oracle_address: Option<&str>,
    pyth_oracle_address: Option<&str>,
) -> InstantiateMsg {
    let native_collateral: AssetInfo = AssetInfo::Native(NATIVE_COLLATERAL_DENOM.to_string());
    let cw20_denom = cw20_address.unwrap_or(CW20_COLLATERAL_DENOM);
    let cw20_collateral: AssetInfo = AssetInfo::Cw20(Addr::unchecked(cw20_denom.to_string()));
    // let assets: Vec<AssetInfo> = vec![native_collateral, cw20_collateral];
    let assets: Vec<AssetInfo> = vec![native_collateral, cw20_collateral];
    let dsc_address = dsc_address.unwrap_or(DSC_ADDR);
    let oracle_address = oracle_address.unwrap_or(ORACLE_ADDRESS);
    let pyth_oracle_address = pyth_oracle_address.unwrap_or(PYTH_ORACLE_ADDRESS);
    let price_feed_ids: Vec<String> =
        vec![String::from(PRICE_FEED_ID_1), String::from(PRICE_FEED_ID_2)];
    InstantiateMsg {
        owner: String::from(OWNER),
        assets: assets,
        oracle_address: String::from(oracle_address),
        pyth_oracle_address: String::from(pyth_oracle_address),
        price_feed_ids: price_feed_ids,
        dsc_address: String::from(dsc_address),
        liquidation_threshold: LIQ_THRESHOLD,
        liquidation_bonus: LIQ_BONUS,
        min_health_factor: MIN_HEALTH_FACTOR,
    }
}

fn get_cw20_instantiate_msg() -> Cw20InstantiateMsg {
    Cw20InstantiateMsg {
        name: String::from("CW20 Token"),
        symbol: String::from(CW20_COLLATERAL_DENOM),
        decimals: 6,
        initial_balances: vec![Cw20Coin {
            address: String::from(OWNER),
            amount: CW20_AMOUNT_MINTED_TO_OWNER,
        }],
        mint: None,
        marketing: None,
    }
}

fn get_dsc_instantiate_msg() -> Cw20InstantiateMsg {
    Cw20InstantiateMsg {
        name: String::from("Decentralized Stablecoin"),
        symbol: String::from("DSC"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: String::from(OWNER),
            cap: None,
        }),
        marketing: None,
    }
}

fn dsc_engine_setup(cw20_address: Option<&str>) -> (App, Addr) {
    let mut app = App::default();
    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id: u64 = app.store_code(Box::new(code));

    let addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(OWNER),
            &get_default_instantiate_msg(cw20_address, None, None, None),
            &[],
            "dsc_engine",
            Some(String::from(OWNER)),
        )
        .unwrap();
    (app, addr)
}

#[test]
fn proper_instantiation() {
    let (app, addr) = dsc_engine_setup(None);

    let config_input = get_default_instantiate_msg(None, None, None, None);

    let config_res: ConfigResponse = app
        .wrap()
        .query_wasm_smart(addr, &QueryMsg::Config {})
        .unwrap();

    let assets_to_feeds: HashMap<String, String> = config_input
        .assets
        .into_iter()
        .map(|asset| asset.inner())
        .zip(config_input.price_feed_ids.into_iter())
        .collect();

    assert_eq!(&config_res.owner, &config_input.owner);
    assert_eq!(config_res.assets_to_feeds, assets_to_feeds);
    assert_eq!(&config_res.oracle_address, &config_input.oracle_address);
    assert_eq!(&config_res.dsc_address, &config_input.dsc_address);
}

#[test]
fn proper_deposit_valid_cw20_collateral_and_mint() {
    let mut app = App::default();

    // 1 - Instantiate mock-pyth and price oracle

    let mock_pyth_code =
        ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
    let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

    let mock_pyth_price_feed_addr = app
        .instantiate_contract(
            mock_pyth_code_id,
            Addr::unchecked(OWNER),
            &Empty {},
            &[],
            "mock-pyth",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
    let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

    let oracle_addr = app
        .instantiate_contract(
            oracle_code_id,
            Addr::unchecked(OWNER),
            &OracleInstantiateMsg {},
            &[],
            "oracle",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let _price: FetchPriceResponse = app
        .wrap()
        .query_wasm_smart(
            oracle_addr.clone(),
            &OracleQueryMsg::FetchPrice {
                pyth_contract_addr: mock_pyth_price_feed_addr.to_string(),
                price_feed_id: PriceIdentifier::from_hex(PRICE_FEED_ID_1).unwrap(),
            },
        )
        .unwrap();

    // 2 - Instantiate cw20 token contract and mint to test user at instantiation

    let cw20_code = ContractWrapper::new(cw20_execute, cw20_instantiate, cw20_query);
    let cw20_code_id: u64 = app.store_code(Box::new(cw20_code));

    let cw20_addr = app
        .instantiate_contract(
            cw20_code_id,
            Addr::unchecked(OWNER),
            &get_cw20_instantiate_msg(),
            &[],
            "cw20",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 3 - Instantiate DSC

    let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
    let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

    let dsc_addr = app
        .instantiate_contract(
            dsc_code_id,
            Addr::unchecked(OWNER),
            &get_dsc_instantiate_msg(),
            &[],
            "dsc",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 4 - Instantiate dsce contract

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id: u64 = app.store_code(Box::new(code));

    let dsce_init_msg = &get_default_instantiate_msg(
        Some(cw20_addr.as_str()),
        Some(dsc_addr.as_str()),
        Some(oracle_addr.as_str()),
        Some(mock_pyth_price_feed_addr.as_str()),
    );

    let dsce_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(OWNER),
            &dsce_init_msg,
            &[],
            "dsc_engine",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let _config_res: ConfigResponse = app
        .wrap()
        .query_wasm_smart(dsce_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    // 5 - Update dsc minter to dsce

    let _update_dsc_minter_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::UpdateMinter {
                new_minter: Some(String::from(dsce_addr.clone())),
            },
            &[],
        )
        .unwrap();

    // 6 - Increase Allowance of cw20 from user to dsce contract

    let _increase_allowance_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: String::from(dsce_addr.clone()),
                amount: CW20_AMOUNT_MINTED_TO_OWNER,
                expires: None,
            },
            &[],
        )
        .unwrap();

    // 7 - execute deposit_collateral_and_mint_dsc

    let initial_deposited_owner_cw20_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(cw20_addr.clone()),
            },
        )
        .unwrap();

    let resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsce_addr.clone(),
            &ExecuteMsg::DepositCollateralAndMintDsc {
                collateral_asset: AssetInfo::Cw20(Addr::unchecked(cw20_addr.as_str())),
                amount_collateral: AMOUNT_COLLATERAL_OK,
                amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
            },
            &[],
        )
        .unwrap();

    // Test deposit events
    let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "action")
            .unwrap()
            .value,
        "deposit_collateral"
    );
    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "from")
            .unwrap()
            .value,
        OWNER
    );
    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "asset")
            .unwrap()
            .value,
        cw20_addr.as_str()
    );
    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "amount")
            .unwrap()
            .value,
        String::from(AMOUNT_COLLATERAL_OK)
    );

    let final_deposited_owner_cw20_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr,
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(cw20_addr),
            },
        )
        .unwrap();

    let final_owner_dsc_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            dsc_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(OWNER),
            },
        )
        .unwrap();

    let final_dsc_info: TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
        .unwrap();

    assert_eq!(initial_deposited_owner_cw20_balance, Uint128::zero());
    assert_eq!(final_deposited_owner_cw20_balance, AMOUNT_COLLATERAL_OK);
    assert_eq!(final_dsc_info.total_supply, AMOUNT_DSC_TO_MINT_OK);
    assert_eq!(final_owner_dsc_balance.balance, AMOUNT_DSC_TO_MINT_OK);
}

#[test]
fn proper_deposit_valid_native_collateral_and_mint() {
    // 0 - Instantiate mock env with native balances associated to OWNER
    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(OWNER),
                coins(INITIAL_OWNER_NATIVE_BALANCE, NATIVE_COLLATERAL_DENOM),
            )
            .unwrap()
    });

    // 1 - Instantiate mock-pyth and price oracle

    let mock_pyth_code =
        ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
    let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

    let mock_pyth_price_feed_addr = app
        .instantiate_contract(
            mock_pyth_code_id,
            Addr::unchecked(OWNER),
            &Empty {},
            &[],
            "mock-pyth",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
    let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

    let oracle_addr = app
        .instantiate_contract(
            oracle_code_id,
            Addr::unchecked(OWNER),
            &OracleInstantiateMsg {},
            &[],
            "oracle",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let _price: FetchPriceResponse = app
        .wrap()
        .query_wasm_smart(
            oracle_addr.clone(),
            &OracleQueryMsg::FetchPrice {
                pyth_contract_addr: mock_pyth_price_feed_addr.to_string(),
                price_feed_id: PriceIdentifier::from_hex(PRICE_FEED_ID_1).unwrap(),
            },
        )
        .unwrap();

    // 2 - Instantiate DSC

    let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
    let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

    let dsc_addr = app
        .instantiate_contract(
            dsc_code_id,
            Addr::unchecked(OWNER),
            &get_dsc_instantiate_msg(),
            &[],
            "dsc",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 3 - Instantiate dsce contract

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id: u64 = app.store_code(Box::new(code));

    let dsce_init_msg = &get_default_instantiate_msg(
        None,
        Some(dsc_addr.as_str()),
        Some(oracle_addr.as_str()),
        Some(mock_pyth_price_feed_addr.as_str()),
    );

    let dsce_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(OWNER),
            &dsce_init_msg,
            &[],
            "dsc_engine",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let _config_res: ConfigResponse = app
        .wrap()
        .query_wasm_smart(dsce_addr.clone(), &QueryMsg::Config {})
        .unwrap();

    // 4 - Update dsc minter to dsce

    let _update_dsc_minter_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::UpdateMinter {
                new_minter: Some(String::from(dsce_addr.clone())),
            },
            &[],
        )
        .unwrap();

    // 5 - execute deposit_collateral_and_mint_dsc

    let initial_deposited_owner_native_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
            },
        )
        .unwrap();

    let _resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsce_addr.clone(),
            &ExecuteMsg::DepositCollateralAndMintDsc {
                collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                amount_collateral: AMOUNT_COLLATERAL_OK,
                amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
            },
            &[Coin {
                denom: String::from(NATIVE_COLLATERAL_DENOM),
                amount: AMOUNT_COLLATERAL_OK,
            }],
        )
        .unwrap();

    let final_deposited_owner_native_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
            },
        )
        .unwrap();

    let final_owner_dsc_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            dsc_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(OWNER),
            },
        )
        .unwrap();

    let final_dsc_info: TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
        .unwrap();

    assert_eq!(initial_deposited_owner_native_balance, Uint128::zero());
    assert_eq!(
        final_deposited_owner_native_balance,
        Uint128::from(AMOUNT_COLLATERAL_OK)
    );
    assert_eq!(final_dsc_info.total_supply, AMOUNT_DSC_TO_MINT_OK);
    assert_eq!(final_owner_dsc_balance.balance, AMOUNT_DSC_TO_MINT_OK);
}

#[test]
fn proper_redeem_valid_cw20_collateral_and_burn_dsc() {
    let mut app = App::default();

    // 1 - Instantiate mock-pyth and price oracle

    let mock_pyth_code =
        ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
    let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

    let mock_pyth_price_feed_addr = app
        .instantiate_contract(
            mock_pyth_code_id,
            Addr::unchecked(OWNER),
            &Empty {},
            &[],
            "mock-pyth",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
    let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

    let oracle_addr = app
        .instantiate_contract(
            oracle_code_id,
            Addr::unchecked(OWNER),
            &OracleInstantiateMsg {},
            &[],
            "oracle",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 2 - Instantiate cw20 token contract and mint to test user at instantiation

    let cw20_code = ContractWrapper::new(cw20_execute, cw20_instantiate, cw20_query);
    let cw20_code_id: u64 = app.store_code(Box::new(cw20_code));

    let cw20_addr = app
        .instantiate_contract(
            cw20_code_id,
            Addr::unchecked(OWNER),
            &get_cw20_instantiate_msg(),
            &[],
            "cw20",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 3 - Instantiate DSC

    let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
    let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

    let dsc_addr = app
        .instantiate_contract(
            dsc_code_id,
            Addr::unchecked(OWNER),
            &get_dsc_instantiate_msg(),
            &[],
            "dsc",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 4 - Instantiate dsce contract

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id: u64 = app.store_code(Box::new(code));

    let dsce_init_msg = &get_default_instantiate_msg(
        Some(cw20_addr.as_str()),
        Some(dsc_addr.as_str()),
        Some(oracle_addr.as_str()),
        Some(mock_pyth_price_feed_addr.as_str()),
    );

    let dsce_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(OWNER),
            &dsce_init_msg,
            &[],
            "dsc_engine",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 5 - Update dsc minter to dsce

    let _update_dsc_minter_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::UpdateMinter {
                new_minter: Some(String::from(dsce_addr.clone())),
            },
            &[],
        )
        .unwrap();

    // 6 - Increase Allowance of cw20 from user to dsce contract

    let _increase_allowance_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: String::from(dsce_addr.clone()),
                amount: CW20_AMOUNT_MINTED_TO_OWNER,
                expires: None,
            },
            &[],
        )
        .unwrap();

    // 7 - execute deposit_collateral_and_mint_dsc

    let _deposit_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsce_addr.clone(),
            &ExecuteMsg::DepositCollateralAndMintDsc {
                collateral_asset: AssetInfo::Cw20(Addr::unchecked(cw20_addr.as_str())),
                amount_collateral: AMOUNT_COLLATERAL_OK,
                amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
            },
            &[],
        )
        .unwrap();

    // 8 - Increase Allowance of dsc (cw20) from user to dsce contract

    let _increase_allowance_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: String::from(dsce_addr.clone()),
                amount: AMOUNT_DSC_TO_MINT_OK,
                expires: None,
            },
            &[],
        )
        .unwrap();

    // 9 - execute reddem_collateral_and_burn_dsc

    let initial_deposited_owner_cw20_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(cw20_addr.clone()),
            },
        )
        .unwrap();

    let resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsce_addr.clone(),
            &ExecuteMsg::RedeemCollateralForDsc {
                collateral_asset: AssetInfo::Cw20(Addr::unchecked(cw20_addr.as_str())),
                amount_collateral: AMOUNT_COLLATERAL_OK,
                amount_dsc_to_burn: AMOUNT_DSC_TO_MINT_OK,
            },
            &[],
        )
        .unwrap();

    // Test deposit events
    let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "action")
            .unwrap()
            .value,
        "redeem_collateral"
    );
    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "from")
            .unwrap()
            .value,
        OWNER
    );
    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "asset")
            .unwrap()
            .value,
        cw20_addr.as_str()
    );
    assert_eq!(
        wasm.attributes
            .iter()
            .find(|attr| attr.key == "amount")
            .unwrap()
            .value,
        String::from(AMOUNT_COLLATERAL_OK)
    );

    let final_deposited_owner_cw20_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr,
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(cw20_addr),
            },
        )
        .unwrap();

    let final_owner_dsc_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            dsc_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(OWNER),
            },
        )
        .unwrap();

    let final_dsc_info: TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
        .unwrap();

    assert_eq!(initial_deposited_owner_cw20_balance, AMOUNT_COLLATERAL_OK);
    assert_eq!(final_deposited_owner_cw20_balance, Uint128::zero());
    assert_eq!(final_dsc_info.total_supply, Uint128::zero());
    assert_eq!(final_owner_dsc_balance.balance, Uint128::zero());
}

#[test]
fn proper_redeem_valid_native_collateral_and_burn_dsc() {
    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(OWNER),
                coins(INITIAL_OWNER_NATIVE_BALANCE, NATIVE_COLLATERAL_DENOM),
            )
            .unwrap()
    });

    // 1 - Instantiate mock-pyth and price oracle

    let mock_pyth_code =
        ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
    let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

    let mock_pyth_price_feed_addr = app
        .instantiate_contract(
            mock_pyth_code_id,
            Addr::unchecked(OWNER),
            &Empty {},
            &[],
            "mock-pyth",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
    let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

    let oracle_addr = app
        .instantiate_contract(
            oracle_code_id,
            Addr::unchecked(OWNER),
            &OracleInstantiateMsg {},
            &[],
            "oracle",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 2 - Instantiate DSC

    let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
    let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

    let dsc_addr = app
        .instantiate_contract(
            dsc_code_id,
            Addr::unchecked(OWNER),
            &get_dsc_instantiate_msg(),
            &[],
            "dsc",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 3 - Instantiate dsce contract

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id: u64 = app.store_code(Box::new(code));

    let dsce_init_msg = &get_default_instantiate_msg(
        None,
        Some(dsc_addr.as_str()),
        Some(oracle_addr.as_str()),
        Some(mock_pyth_price_feed_addr.as_str()),
    );

    let dsce_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(OWNER),
            &dsce_init_msg,
            &[],
            "dsc_engine",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 4 - Update dsc minter to dsce

    let _update_dsc_minter_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::UpdateMinter {
                new_minter: Some(String::from(dsce_addr.clone())),
            },
            &[],
        )
        .unwrap();

    // 5 - execute deposit_collateral_and_mint_dsc

    let _deposit_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsce_addr.clone(),
            &ExecuteMsg::DepositCollateralAndMintDsc {
                collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                amount_collateral: AMOUNT_COLLATERAL_OK,
                amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
            },
            &[Coin {
                denom: String::from(NATIVE_COLLATERAL_DENOM),
                amount: AMOUNT_COLLATERAL_OK,
            }],
        )
        .unwrap();

    let initial_deposited_owner_native_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
            },
        )
        .unwrap();

    // 6 - Increase Allowance of dsc (cw20) from user to dsce contract

    let _increase_allowance_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: String::from(dsce_addr.clone()),
                amount: AMOUNT_DSC_TO_MINT_OK,
                expires: None,
            },
            &[],
        )
        .unwrap();

    // 7 - execute redeem_collateral_and_burn_dsc

    let _resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsce_addr.clone(),
            &ExecuteMsg::RedeemCollateralForDsc {
                collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                amount_collateral: AMOUNT_COLLATERAL_OK,
                amount_dsc_to_burn: AMOUNT_DSC_TO_MINT_OK,
            },
            &[],
        )
        .unwrap();

    let final_deposited_owner_native_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
            },
        )
        .unwrap();

    let final_owner_dsc_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            dsc_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(OWNER),
            },
        )
        .unwrap();

    let final_dsc_info: TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
        .unwrap();

    assert_eq!(initial_deposited_owner_native_balance, AMOUNT_COLLATERAL_OK);
    assert_eq!(final_deposited_owner_native_balance, Uint128::zero());
    assert_eq!(final_dsc_info.total_supply, Uint128::zero());
    assert_eq!(final_owner_dsc_balance.balance, Uint128::zero());
}

#[test]
fn proper_native_liquidation() {
    let mut app = App::new(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(OWNER),
                coins(INITIAL_OWNER_NATIVE_BALANCE, NATIVE_COLLATERAL_DENOM),
            )
            .unwrap()
    });

    let _send_msg = app.send_tokens(
        Addr::unchecked(OWNER),
        Addr::unchecked(LIQUIDATOR),
        &[Coin {
            denom: String::from(NATIVE_COLLATERAL_DENOM),
            amount: FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR,
        }],
    );

    // 1 - Instantiate mock-pyth and price oracle

    let mock_pyth_code =
        ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
    let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

    let mock_pyth_price_feed_addr = app
        .instantiate_contract(
            mock_pyth_code_id,
            Addr::unchecked(OWNER),
            &Empty {},
            &[],
            "mock-pyth",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
    let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

    let oracle_addr = app
        .instantiate_contract(
            oracle_code_id,
            Addr::unchecked(OWNER),
            &OracleInstantiateMsg {},
            &[],
            "oracle",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 2 - Instantiate DSC

    let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
    let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

    let dsc_addr = app
        .instantiate_contract(
            dsc_code_id,
            Addr::unchecked(OWNER),
            &get_dsc_instantiate_msg(),
            &[],
            "dsc",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 3 - Instantiate dsce contract

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id: u64 = app.store_code(Box::new(code));

    let dsce_init_msg = &get_default_instantiate_msg(
        None,
        Some(dsc_addr.as_str()),
        Some(oracle_addr.as_str()),
        Some(mock_pyth_price_feed_addr.as_str()),
    );

    let dsce_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(OWNER),
            &dsce_init_msg,
            &[],
            "dsc_engine",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 4 - Update dsc minter to dsce

    let _update_dsc_minter_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::UpdateMinter {
                new_minter: Some(String::from(dsce_addr.clone())),
            },
            &[],
        )
        .unwrap();

    // 5 - execute deposit_collateral_and_mint_dsc by user that will be liquidated

    let _deposit_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsce_addr.clone(),
            &ExecuteMsg::DepositCollateralAndMintDsc {
                collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                amount_collateral: AMOUNT_COLLATERAL_OK,
                amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
            },
            &[Coin {
                denom: String::from(NATIVE_COLLATERAL_DENOM),
                amount: AMOUNT_COLLATERAL_OK,
            }],
        )
        .unwrap();

    let _initial_deposited_owner_native_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
            },
        )
        .unwrap();

    // 6 - Increase Allowance of dsc (cw20) from liquidator to dsce contract

    let _increase_allowance_resp = app
        .execute_contract(
            Addr::unchecked(LIQUIDATOR),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: String::from(dsce_addr.clone()),
                amount: AMOUNT_DSC_TO_MINT_OK,
                expires: None,
            },
            &[],
        )
        .unwrap();

    // 7 - Change mock-pyth collateral price - set new lower mocked price

    let _update_mock_price_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            mock_pyth_price_feed_addr.clone(),
            &MockPythExecuteMsg::UpdateMockPrice {
                price: LIQUIDATION_PRICE,
            },
            &[],
        )
        .unwrap();

    // 9 - execute deposit_collateral_and_mint_dsc by liquidator

    let _liquidator_deposit_resp = app
        .execute_contract(
            Addr::unchecked(LIQUIDATOR),
            dsce_addr.clone(),
            &ExecuteMsg::DepositCollateralAndMintDsc {
                collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                amount_collateral: FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR,
                amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
            },
            &[Coin {
                denom: String::from(NATIVE_COLLATERAL_DENOM),
                amount: FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR,
            }],
        )
        .unwrap();

    // 10 - liquidate

    let _liquidation_resp = app
        .execute_contract(
            Addr::unchecked(LIQUIDATOR),
            dsce_addr.clone(),
            &ExecuteMsg::Liquidate {
                collateral_asset: AssetInfo::Native(String::from(NATIVE_COLLATERAL_DENOM)),
                user: String::from(OWNER),
                debt_to_cover: Decimal::from_atomics(DEBT_TO_COVER, 6).unwrap(),
            },
            &[],
        )
        .unwrap();

    // Assert final state

    let final_deposited_owner_native_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
            },
        )
        .unwrap();

    let final_deposited_liquidator_native_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(LIQUIDATOR),
                collateral_asset: String::from(NATIVE_COLLATERAL_DENOM),
            },
        )
        .unwrap();

    let final_owner_dsc_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            dsc_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(OWNER),
            },
        )
        .unwrap();

    let final_liquidator_dsc_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            dsc_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(LIQUIDATOR),
            },
        )
        .unwrap();

    let final_owner_native_balance: Uint128 = app
        .wrap()
        .query_balance(OWNER, NATIVE_COLLATERAL_DENOM)
        .unwrap()
        .amount;

    let final_liquidator_native_balance: Uint128 = app
        .wrap()
        .query_balance(LIQUIDATOR, NATIVE_COLLATERAL_DENOM)
        .unwrap()
        .amount;

    let final_dsc_info: TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
        .unwrap();

    assert_eq!(
        final_deposited_owner_native_balance,
        FINAL_COLLATERAL_BALANCE_OF_LIQUIDATED
    );
    assert_eq!(
        final_deposited_liquidator_native_balance,
        FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR
    );
    assert_eq!(
        final_owner_dsc_balance.balance,
        FINAL_DSC_BALANCE_OF_LIQUIDATED
    );
    assert_eq!(
        final_liquidator_dsc_balance.balance,
        FINAL_DSC_BALANCE_OF_LIQUIDATOR
    );
    assert_eq!(
        final_owner_native_balance,
        FINAL_NATIVE_BALANCE_OF_LIQUIDATED
    );
    assert_eq!(final_liquidator_native_balance, FINAL_BALANCE_OF_LIQUIDATOR);
    assert_eq!(final_dsc_info.total_supply, FINAL_DSC_SUPPLY);
}

#[test]
fn proper_cw20_liquidation() {
    let mut app = App::default();

    // 1 - Instantiate cw20 token contract and mint to test users at instantiation

    let cw20_code = ContractWrapper::new(cw20_execute, cw20_instantiate, cw20_query);
    let cw20_code_id: u64 = app.store_code(Box::new(cw20_code));

    let cw20_instantiate_msg = Cw20InstantiateMsg {
        name: String::from("CW20 Token"),
        symbol: String::from(CW20_COLLATERAL_DENOM),
        decimals: 6,
        initial_balances: vec![
            Cw20Coin {
                address: String::from(OWNER),
                amount: Uint128::from(INITIAL_OWNER_NATIVE_BALANCE),
            },
            Cw20Coin {
                address: String::from(LIQUIDATOR),
                amount: FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR,
            },
        ],
        mint: None,
        marketing: None,
    };

    let cw20_addr = app
        .instantiate_contract(
            cw20_code_id,
            Addr::unchecked(OWNER),
            &cw20_instantiate_msg,
            &[],
            "cw20",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 2 - Instantiate mock-pyth and price oracle

    let mock_pyth_code =
        ContractWrapper::new(mock_pyth_execute, mock_pyth_instantiate, mock_pyth_query);
    let mock_pyth_code_id: u64 = app.store_code(Box::new(mock_pyth_code));

    let mock_pyth_price_feed_addr = app
        .instantiate_contract(
            mock_pyth_code_id,
            Addr::unchecked(OWNER),
            &Empty {},
            &[],
            "mock-pyth",
            Some(String::from(OWNER)),
        )
        .unwrap();

    let oracle_code = ContractWrapper::new(oracle_execute, oracle_instantiate, oracle_query);
    let oracle_code_id: u64 = app.store_code(Box::new(oracle_code));

    let oracle_addr = app
        .instantiate_contract(
            oracle_code_id,
            Addr::unchecked(OWNER),
            &OracleInstantiateMsg {},
            &[],
            "oracle",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 3 - Instantiate DSC

    let dsc_code = ContractWrapper::new(dsc_execute, dsc_instantiate, dsc_query);
    let dsc_code_id: u64 = app.store_code(Box::new(dsc_code));

    let dsc_addr = app
        .instantiate_contract(
            dsc_code_id,
            Addr::unchecked(OWNER),
            &get_dsc_instantiate_msg(),
            &[],
            "dsc",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 4 - Instantiate DSCE

    let code = ContractWrapper::new(execute, instantiate, query);
    let code_id: u64 = app.store_code(Box::new(code));

    let dsce_init_msg = &get_default_instantiate_msg(
        Some(cw20_addr.as_str()),
        Some(dsc_addr.as_str()),
        Some(oracle_addr.as_str()),
        Some(mock_pyth_price_feed_addr.as_str()),
    );

    let dsce_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(OWNER),
            &dsce_init_msg,
            &[],
            "dsc_engine",
            Some(String::from(OWNER)),
        )
        .unwrap();

    // 5 - Update DSC minter to DSCE

    let _update_dsc_minter_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::UpdateMinter {
                new_minter: Some(String::from(dsce_addr.clone())),
            },
            &[],
        )
        .unwrap();

    // 6 - Increase Allowance of cw20 from users (owner + liquidator) to dsce contract

    let _increase_allowance_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: String::from(dsce_addr.clone()),
                amount: CW20_AMOUNT_MINTED_TO_OWNER,
                expires: None,
            },
            &[],
        )
        .unwrap();

    let _increase_allowance_resp = app
        .execute_contract(
            Addr::unchecked(LIQUIDATOR),
            cw20_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: String::from(dsce_addr.clone()),
                amount: CW20_AMOUNT_MINTED_TO_OWNER,
                expires: None,
            },
            &[],
        )
        .unwrap();

    // 7 - execute deposit_collateral_and_mint_dsc by user that will be liquidated

    let _deposit_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            dsce_addr.clone(),
            &ExecuteMsg::DepositCollateralAndMintDsc {
                collateral_asset: AssetInfo::Cw20(Addr::unchecked(cw20_addr.as_str())),
                amount_collateral: AMOUNT_COLLATERAL_OK,
                amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
            },
            &[],
        )
        .unwrap();

    let _initial_deposited_owner_cw20_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(cw20_addr.clone()),
            },
        )
        .unwrap();

    // 8 - Increase Allowance of dsc (cw20) from liquidator to dsce contract

    let _increase_allowance_resp = app
        .execute_contract(
            Addr::unchecked(LIQUIDATOR),
            dsc_addr.clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: String::from(dsce_addr.clone()),
                amount: AMOUNT_DSC_TO_MINT_OK,
                expires: None,
            },
            &[],
        )
        .unwrap();

    // 9 - Change mock-pyth collateral price - set new lower mocked price

    let _update_mock_price_resp = app
        .execute_contract(
            Addr::unchecked(OWNER),
            mock_pyth_price_feed_addr.clone(),
            &MockPythExecuteMsg::UpdateMockPrice {
                price: LIQUIDATION_PRICE,
            },
            &[],
        )
        .unwrap();

    // 10 - execute deposit_collateral_and_mint_dsc by liquidator

    let _liquidator_deposit_resp = app
        .execute_contract(
            Addr::unchecked(LIQUIDATOR),
            dsce_addr.clone(),
            &ExecuteMsg::DepositCollateralAndMintDsc {
                collateral_asset: AssetInfo::Cw20(Addr::unchecked(cw20_addr.as_str())),
                amount_collateral: FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR,
                amount_dsc_to_mint: AMOUNT_DSC_TO_MINT_OK,
            },
            &[],
        )
        .unwrap();

    // 11 - liquidate

    let _liquidation_resp = app
        .execute_contract(
            Addr::unchecked(LIQUIDATOR),
            dsce_addr.clone(),
            &ExecuteMsg::Liquidate {
                collateral_asset: AssetInfo::Cw20(Addr::unchecked(cw20_addr.as_str())),
                user: String::from(OWNER),
                debt_to_cover: Decimal::from_atomics(DEBT_TO_COVER, 6).unwrap(),
            },
            &[],
        )
        .unwrap();

    // Assert final state

    let final_deposited_owner_cw20_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(OWNER),
                collateral_asset: String::from(cw20_addr.clone()),
            },
        )
        .unwrap();

    let final_deposited_liquidator_cw20_balance: Uint128 = app
        .wrap()
        .query_wasm_smart(
            dsce_addr.clone(),
            &QueryMsg::CollateralBalanceOfUser {
                user: String::from(LIQUIDATOR),
                collateral_asset: String::from(cw20_addr.clone()),
            },
        )
        .unwrap();

    let final_owner_dsc_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            dsc_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(OWNER),
            },
        )
        .unwrap();

    let final_liquidator_dsc_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            dsc_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(LIQUIDATOR),
            },
        )
        .unwrap();

    let final_owner_cw20_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(OWNER),
            },
        )
        .unwrap();

    let final_liquidator_cw20_balance: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            cw20_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: String::from(LIQUIDATOR),
            },
        )
        .unwrap();

    let final_dsc_info: TokenInfoResponse = app
        .wrap()
        .query_wasm_smart(dsc_addr, &Cw20QueryMsg::TokenInfo {})
        .unwrap();

    assert_eq!(
        final_deposited_owner_cw20_balance,
        FINAL_COLLATERAL_BALANCE_OF_LIQUIDATED
    );
    assert_eq!(
        final_deposited_liquidator_cw20_balance,
        FINAL_COLLATERAL_BALANCE_OF_LIQUIDATOR
    );
    assert_eq!(
        final_owner_dsc_balance.balance,
        FINAL_DSC_BALANCE_OF_LIQUIDATED
    );
    assert_eq!(
        final_liquidator_dsc_balance.balance,
        FINAL_DSC_BALANCE_OF_LIQUIDATOR
    );
    assert_eq!(
        final_owner_cw20_balance.balance,
        FINAL_CW20_BALANCE_OF_LIQUIDATED
    );
    assert_eq!(
        final_liquidator_cw20_balance.balance,
        FINAL_BALANCE_OF_LIQUIDATOR
    );
    assert_eq!(final_dsc_info.total_supply, FINAL_DSC_SUPPLY);
}
