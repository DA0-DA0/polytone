use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Hello { data: Option<Binary> },
    Callback(polytone::callback::CallbackMessage),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CallbackHistoryResponse)]
    History {},
}

#[cw_serde]
pub struct CallbackHistoryResponse {
    pub history: Vec<polytone::callback::CallbackMessage>,
}
