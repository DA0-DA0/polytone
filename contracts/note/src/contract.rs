#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use polytone::callback::CallbackRequestType;
use polytone::{callback, ibc};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, Pair, QueryMsg};
use crate::state::{CHANNEL, CONNECTION_REMOTE_PORT, CONTROLLER};

const CONTRACT_NAME: &str = "crates.io:polytone-note";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let mut response = Response::default().add_attribute("method", "instantiate");

    if let Some(Pair {
        connection_id,
        remote_port,
    }) = msg.pair
    {
        response = response
            .add_attribute("pair_connection", connection_id.to_string())
            .add_attribute("pair_port", remote_port.to_string());
        CONNECTION_REMOTE_PORT.save(deps.storage, &(connection_id, remote_port))?;
    };

    if let Some(controller) = msg.controller {
        response = response.add_attribute("controller", controller.to_string());
        CONTROLLER.save(deps.storage, &deps.api.addr_validate(&controller)?)?;
    }
    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let (msg, callback, timeout_seconds, request_type, on_behalf_of) = match msg {
        ExecuteMsg::Execute {
            msgs,
            callback,
            timeout_seconds,
            on_behalf_of,
        } => (
            ibc::Msg::Execute { msgs },
            callback,
            timeout_seconds,
            CallbackRequestType::Execute,
            on_behalf_of,
        ),
        ExecuteMsg::Query {
            msgs,
            callback,
            timeout_seconds,
            on_behalf_of,
        } => (
            ibc::Msg::Query { msgs },
            Some(callback),
            timeout_seconds,
            CallbackRequestType::Query,
            on_behalf_of,
        ),
    };

    // Check if the note is controlled
    let sender = match CONTROLLER.load(deps.storage) {
        Ok(controller) => {
            if controller != info.sender {
                return Err(ContractError::NotController);
            }

            // Note is controlled and controller is the sender
            // now verify we have on_behalf_of set to know who
            // the real sender is
            if let Some(sender) = on_behalf_of {
                deps.api.addr_validate(&sender)
            } else {
                // Note is controlled, but on_behalf_of is not set
                return Err(ContractError::OnBehalfOfNotSet);
            }
        }
        // Note is not controlled, but on_behalf_of is set
        // probably something was misconfigured
        Err(_) => {
            if on_behalf_of.is_some() {
                return Err(ContractError::NotControlledButOnBehalfIsSet);
            }
            Ok(info.sender)
        }
    }?;

    callback::request_callback(
        deps.storage,
        deps.api,
        sender.clone(),
        callback,
        request_type,
    )?;

    let channel_id = CHANNEL
        .may_load(deps.storage)?
        .ok_or(ContractError::NoPair)?;
    Ok(Response::default()
        .add_attribute("method", "execute")
        .add_message(IbcMsg::SendPacket {
            channel_id,
            data: to_binary(&ibc::Packet {
                sender: sender.into_string(),
                msg,
            })
            .expect("msgs are known to be serializable"),
            timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(timeout_seconds.u64())),
        }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ActiveChannel => to_binary(&CHANNEL.may_load(deps.storage)?),
        QueryMsg::Pair => to_binary(&CONNECTION_REMOTE_PORT.may_load(deps.storage)?.map(
            |(connection_id, remote_port)| Pair {
                connection_id,
                remote_port,
            },
        )),
        QueryMsg::RemoteAddress { local_address } => to_binary(
            &callback::LOCAL_TO_REMOTE_ACCOUNT
                .may_load(deps.storage, &deps.api.addr_validate(&local_address)?)?,
        ),
    }
}
