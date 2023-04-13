#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg,
    SubMsgResponse, SubMsgResult,
};
use cw2::set_contract_version;
use polytone::ack::ack_execute_success;
use polytone::error_reply::ErrorReply;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{COLLECTOR, INSTANTIATOR};

const CONTRACT_NAME: &str = "crates.io:polytone-proxy";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    INSTANTIATOR.save(deps.storage, &info.sender)?;

    Ok(Response::default()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Proxy { msgs } => {
            if info.sender == INSTANTIATOR.load(deps.storage)? {
                COLLECTOR.save(deps.storage, &vec![None; msgs.len()])?;
                Ok(Response::default()
                    .add_attribute("method", "execute_proxy")
                    .add_attribute("sender", info.sender)
                    .add_submessages(
                        msgs.into_iter()
                            .enumerate()
                            .map(|(id, msg)| SubMsg::reply_always(msg, id as u64)),
                    )
                    // handle `msgs.is_empty()` case
                    .set_data(ack_execute_success(
                        vec![],
                        env.contract.address.into_string(),
                    )))
            } else {
                Err(ContractError::NotInstantiator)
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Instantiator {} => to_binary(&INSTANTIATOR.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut collector = COLLECTOR.load(deps.storage)?;

    match msg.result {
        SubMsgResult::Err(error) => Err(ContractError::Reply(ErrorReply::new(msg.id, error))),
        SubMsgResult::Ok(res) => {
            collector[msg.id as usize] = Some(res);

            if msg.id + 1 == collector.len() as u64 {
                COLLECTOR.remove(deps.storage);
                let collector = collector
                    .into_iter()
                    .map(|res| res.unwrap())
                    .collect::<Vec<SubMsgResponse>>();
                Ok(Response::default()
                    .add_attribute("callbacks_processed", (msg.id + 1).to_string())
                    .set_data(ack_execute_success(
                        collector,
                        env.contract.address.into_string(),
                    )))
            } else {
                COLLECTOR.save(deps.storage, &collector)?;
                Ok(Response::default())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        from_binary,
        testing::{mock_dependencies, mock_env},
        Uint64,
    };

    use polytone::callback::ErrorResponse;

    use super::*;

    /// When we are returning an execution failure, the error string
    /// should be exactly the same as the binary representation of a
    /// `polytone::callbck::ErrorResponse`.
    #[test]
    fn test_error_serialization() {
        let mut deps = mock_dependencies();
        COLLECTOR
            .save(deps.as_mut().storage, &vec![None; 1])
            .unwrap();
        let error = reply(
            deps.as_mut(),
            mock_env(),
            Reply {
                id: 1,
                result: SubMsgResult::Err("hello".to_string()),
            },
        )
        .unwrap_err()
        .to_string();
        let error_response = Binary::from_base64(&error).unwrap();
        let error_response: ErrorResponse = from_binary(&error_response).unwrap();
        assert_eq!(
            error_response,
            ErrorResponse {
                message_index: Uint64::new(1),
                error: "hello".to_string()
            }
        )
    }
}
