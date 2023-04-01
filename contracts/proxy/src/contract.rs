#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult,
    SubMsg, SubMsgResult,
};
use cw2::set_contract_version;

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
    _env: Env,
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
                    ))
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
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut collector = COLLECTOR.load(deps.storage)?;

    match msg.result {
        SubMsgResult::Err(error) => Err(ContractError::ExecutionFailure { idx: msg.id, error }),
        SubMsgResult::Ok(res) => {
            collector[msg.id as usize] = Some(res);

            if msg.id + 1 == collector.len() as u64 {
                COLLECTOR.remove(deps.storage);
                // Unwrap the options as we set it to Some
                let collector: Vec<Binary> = collector
                    .into_iter()
                    .map(|res| to_binary(&res.unwrap()))
                    .collect::<Result<Vec<Binary>, StdError>>()?;
                Ok(Response::default()
                    .add_attribute("callbacks_processed", (msg.id + 1).to_string())
                    .set_data(to_binary(&collector)?))
            } else {
                COLLECTOR.save(deps.storage, &collector)?;
                Ok(Response::default())
            }
        }
    }
}
