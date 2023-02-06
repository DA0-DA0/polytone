#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    DepsMut, Env, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg,
    IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, Never, Reply, Response, SubMsg, SubMsgResult,
};
use polytone::{callback, ibc::validate_order_and_version};

use crate::{
    error::ContractError,
    state::{CHANNEL, CONNECTION_REMOTE_PORT},
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    let (channel, counterparty_version) = icom(msg);
    validate_order_and_version(&channel, counterparty_version.as_deref())?;
    match CONNECTION_REMOTE_PORT.may_load(deps.storage)? {
        Some((conn, port)) => {
            if channel.counterparty_endpoint.port_id != port || channel.connection_id != conn {
                Err(ContractError::AlreadyPaired {
                    suggested_connection: channel.connection_id,
                    suggested_port: channel.counterparty_endpoint.port_id,
                    pair_connection: conn,
                    pair_port: port,
                })
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let (channel, counterparty_version) = iccm(msg);
    validate_order_and_version(&channel, counterparty_version.as_deref())?;
    CONNECTION_REMOTE_PORT.save(
        deps.storage,
        &(channel.connection_id, channel.counterparty_endpoint.port_id),
    )?;
    CHANNEL.save(deps.storage, &channel.endpoint.channel_id)?;
    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_connect")
        .add_attribute("channel_id", channel.endpoint.channel_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    CHANNEL.remove(deps.storage);
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
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    unreachable!("voice should never send a packet")
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    ack: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let callback = callback::on_ack(deps.storage, &ack);
    Ok(IbcBasicResponse::default()
        .add_attribute("method", "ibc_packet_ack")
        .add_attribute("sequence_number", ack.original_packet.sequence.to_string())
        .add_submessages(callback.map(|c| SubMsg::reply_on_error(c, ack.original_packet.sequence))))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let callback = callback::on_timeout(deps.storage, &msg);
    Ok(IbcBasicResponse::default()
        .add_attribute("method", "ibc_packet_timeout")
        .add_attribute("sequence_number", msg.packet.sequence.to_string())
        .add_submessages(callback.map(|c| SubMsg::reply_on_error(c, msg.packet.sequence))))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let sequence = msg.id;
    match msg.result {
        SubMsgResult::Err(e) => Ok(Response::default()
            .add_attribute("callback_error", sequence.to_string())
            .add_attribute("error", e)),
        SubMsgResult::Ok(_) => unreachable!("callbacks reply_on_error"),
    }
}

fn icom(msg: IbcChannelOpenMsg) -> (IbcChannel, Option<String>) {
    match msg {
        IbcChannelOpenMsg::OpenInit { channel } => (channel, None),
        IbcChannelOpenMsg::OpenTry {
            channel,
            counterparty_version,
        } => (channel, Some(counterparty_version)),
    }
}

fn iccm(msg: IbcChannelConnectMsg) -> (IbcChannel, Option<String>) {
    match msg {
        IbcChannelConnectMsg::OpenAck {
            channel,
            counterparty_version,
        } => (channel, Some(counterparty_version)),
        IbcChannelConnectMsg::OpenConfirm { channel } => (channel, None),
    }
}
