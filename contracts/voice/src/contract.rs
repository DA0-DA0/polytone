#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, instantiate2_address, to_binary, to_vec, Binary, CodeInfoResponse, ContractResult,
    Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, SubMsg, SystemResult, WasmMsg,
};
use cw2::set_contract_version;

use polytone::ibc::{Msg, Packet};

use crate::error::ContractError;
use crate::ibc::REPLY_FORWARD_DATA;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{PROXY_CODE_ID, SENDER_TO_PROXY};

const CONTRACT_NAME: &str = "crates.io:polytone-voice";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    PROXY_CODE_ID.save(deps.storage, &msg.proxy_code_id.u64())?;
    Ok(Response::default().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Rx {
            connection_id,
            counterparty_port,
            data,
        } => {
            if info.sender != env.contract.address {
                Err(ContractError::NotSelf)
            } else {
                let Packet { sender, msg } = from_binary(&data)?;
                match msg {
                    Msg::Query { msgs } => {
                        let mut results = Vec::with_capacity(msgs.len());
                        for msg in msgs {
                            let qr = deps.querier.raw_query(&to_vec(&msg)?);
                            match qr {
                                SystemResult::Ok(ContractResult::Err(e)) => {
                                    return Err(StdError::generic_err(format!(
                                        "query contract: {e}"
                                    ))
                                    .into())
                                }
                                SystemResult::Err(e) => {
                                    return Err(
                                        StdError::generic_err(format!("query system: {e}")).into()
                                    )
                                }
                                SystemResult::Ok(ContractResult::Ok(res)) => results.push(res),
                            }
                        }
                        Ok(Response::default()
                            .add_attribute("method", "rx_query")
                            .set_data(to_binary(&results)?))
                    }
                    Msg::Execute { msgs } => {
                        let (instantiate, proxy) = if let Some(proxy) = SENDER_TO_PROXY.may_load(
                            deps.storage,
                            (connection_id, counterparty_port, sender.clone()),
                        )? {
                            (None, proxy)
                        } else {
                            let contract =
                                deps.api.addr_canonicalize(env.contract.address.as_str())?;
                            let code_id = PROXY_CODE_ID.load(deps.storage)?;
                            let CodeInfoResponse { checksum, .. } =
                                deps.querier.query_wasm_code_info(code_id)?;
                            let salt = Binary::from(sender.as_bytes());
                            let proxy = deps.api.addr_humanize(&instantiate2_address(
                                &checksum, &contract, &salt,
                            )?)?;
                            (
                                Some(WasmMsg::Instantiate2 {
                                    admin: None,
                                    code_id,
                                    label: format!("polytone-proxy {sender}"),
                                    msg: to_binary(&polytone_proxy::msg::InstantiateMsg {})?,
                                    funds: vec![],
                                    salt,
                                }),
                                proxy,
                            )
                        };
                        Ok(Response::default()
                            .add_attribute("method", "rx_execute")
                            .add_messages(instantiate)
                            .add_submessage(SubMsg::reply_on_success(
                                WasmMsg::Execute {
                                    contract_addr: proxy.into_string(),
                                    msg: to_binary(&polytone_proxy::msg::ExecuteMsg::Proxy {
                                        msgs,
                                    })?,
                                    funds: vec![],
                                },
                                REPLY_FORWARD_DATA,
                            )))
                    }
                }
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
