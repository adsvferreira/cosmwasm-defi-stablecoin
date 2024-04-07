use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum ExecuteMsg {
    UpdateMockPrice { price: i64 },
}
