use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint64};

#[cw_serde]
pub struct InstantiateMsg {
    pub proxy_code_id: Uint64,
    pub block_max_gas: Uint64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Rx {
        connection_id: String,
        counterparty_port: String,
        data: Binary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
