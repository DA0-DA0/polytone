#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, DepsMut, Env, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Never, Reply, Response, SubMsg,
    SubMsgResult, WasmMsg,
};

use cw_utils::{parse_reply_execute_data, MsgExecuteContractResponse};
use polytone::{
    ack::{ack_fail, ack_success},
    ibc::validate_order_and_version,
};

use crate::{error::ContractError, msg::ExecuteMsg, state::CHANNEL_TO_CONNECTION};

const REPLY_ACK: u64 = 0;
/// If more than one messages are dispatched from a message, data set
/// by those messages will not be automatically percolated up. If
/// there is a single message, it will be.
pub(crate) const REPLY_FORWARD_DATA: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    CHANNEL_TO_CONNECTION.save(
        deps.storage,
        msg.channel().endpoint.channel_id.clone(),
        &msg.channel().connection_id,
    )?;
    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_connect")
        .add_attribute("channel_id", msg.channel().endpoint.channel_id.clone()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    CHANNEL_TO_CONNECTION.remove(deps.storage, msg.channel().endpoint.channel_id.clone());
    Ok(IbcBasicResponse::default()
        .add_attribute("method", "ibc_channel_close")
        .add_attribute("connection_id", msg.channel().connection_id.clone())
        .add_attribute(
            "counterparty_port_id",
            msg.channel().counterparty_endpoint.port_id.clone(),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    Ok(
        IbcReceiveResponse::default().add_submessage(SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: env.contract.address.into_string(),
                msg: to_binary(&ExecuteMsg::Rx {
                    connection_id: CHANNEL_TO_CONNECTION
                        .load(deps.storage, msg.packet.dest.channel_id)
                        .expect("handshake sets mapping"),
                    counterparty_port: msg.packet.src.port_id,
                    data: msg.packet.data,
                })
                .unwrap(),
                funds: vec![],
            },
            REPLY_ACK,
        )),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        REPLY_ACK => Ok(match msg.result {
            SubMsgResult::Err(e) => Response::default()
                .add_attribute("ack_error", &e)
                .set_data(ack_fail(e)),
            SubMsgResult::Ok(_) => {
                let data = parse_reply_execute_data(msg)
                    .expect("execution succeded")
                    .data
                    .expect("proxy should set data");
                match from_binary::<Vec<Option<Binary>>>(&data) {
                    Ok(d) => Response::default().set_data(ack_success(d)),
                    Err(e) => Response::default()
                        .set_data(ack_fail(format!("unmarshaling callback data: ({e})"))),
                }
            }
        }),
        REPLY_FORWARD_DATA => {
            let MsgExecuteContractResponse { data } = parse_reply_execute_data(msg)?;
            let response = Response::default().add_attribute("method", "reply_forward_data");
            Ok(match data {
                Some(data) => response.set_data(data),
                None => response,
            })
        }
        _ => unreachable!("unknown reply ID"),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _ack: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    unreachable!("host will never send a packet")
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    unreachable!("host will never send a packet")
}
