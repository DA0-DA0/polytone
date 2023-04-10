use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, Api, Binary, CosmosMsg, IbcPacketAckMsg, IbcPacketTimeoutMsg, StdResult,
    Storage, WasmMsg,
};
use cw_storage_plus::{Item, Map};

use crate::ack::unmarshal_ack;

/// Executed on the callback receiver upon message completion. When
/// being executed, the message will be tagged with "callback":
///
/// ```json
/// {"callback": {
///       "initiator": ...,
///       "initiator_msg": ...,
///       "result": ...,
/// }}
/// ```
#[cw_serde]
pub struct CallbackMessage {
    /// Initaitor on the controller chain.
    pub initiator: Addr,
    /// Message sent by the initaitor. This _must_ be base64 encoded
    /// or execution will fail.
    pub initiator_msg: Binary,
    /// Data from the host chain.
    pub result: Callback,
}

#[cw_serde]
pub enum Callback {
    /// Data returned from the host chain. Index n corresponds to the
    /// result of executing the nth message/query.
    Success(Vec<Option<Binary>>),
    /// The first error that occured while executing the requested
    /// messages/queries.
    Error(String),
}

/// Serialized into the error string in the event of a timeout sending
/// the message.
#[cw_serde]
pub enum Timeout {
    Timeout,
}

/// A request for a callback. Safe for use in external APIs.
#[cw_serde]
pub struct CallbackRequest {
    pub receiver: String,
    pub msg: Binary,
}

//// Must be called every time a packet is sent to keep sequence
//// number tracking accurate.
///
/// Requests that a callback be returned for the next IBC message that
/// is sent. `on_ibc_send` must be called before this method in
/// any functions that send IBC packets.
pub fn request_callback(
    storage: &mut dyn Storage,
    api: &dyn Api,
    initiator: Addr,
    request: Option<CallbackRequest>,
) -> StdResult<()> {
    let seq = SEQ.may_load(storage)?.unwrap_or_default() + 1;
    SEQ.save(storage, &seq)?;

    if let Some(request) = request {
        let receiver = api.addr_validate(&request.receiver)?;
        let initiator_msg = request.msg;

        CALLBACKS.save(
            storage,
            seq,
            &PendingCallback {
                initiator,
                initiator_msg,
                receiver,
            },
        )?;
    }

    Ok(())
}

fn callback_msg(pc: PendingCallback, c: Callback) -> CosmosMsg {
    /// Gives the executed message a "callback" tag:
    /// `{ "callback": CallbackMsg }`.
    #[cw_serde]
    enum C {
        Callback(CallbackMessage),
    }
    WasmMsg::Execute {
        contract_addr: pc.receiver.into_string(),
        msg: to_binary(&C::Callback(CallbackMessage {
            initiator: pc.initiator,
            initiator_msg: pc.initiator_msg,
            result: c,
        }))
        .expect("fields are known to be serializable"),
        funds: vec![],
    }
    .into()
}

/// Call on every packet ACK. Returns a callback message to execute,
/// if any.
pub fn on_ack(
    storage: &mut dyn Storage,
    IbcPacketAckMsg {
        acknowledgement,
        original_packet,
        ..
    }: &IbcPacketAckMsg,
) -> Option<CosmosMsg> {
    let Some(request) = CALLBACKS.may_load(storage, original_packet.sequence).unwrap() else {
	return None
    };
    CALLBACKS.remove(storage, original_packet.sequence);
    let result = unmarshal_ack(acknowledgement);
    Some(callback_msg(request, result))
}

/// Call on every packet timeout. Returns a callback message to execute,
/// if any.
pub fn on_timeout(
    storage: &mut dyn Storage,
    IbcPacketTimeoutMsg { packet, .. }: &IbcPacketTimeoutMsg,
) -> Option<CosmosMsg> {
    let Some(request) = CALLBACKS.may_load(storage, packet.sequence).unwrap() else {
	return None
    };
    CALLBACKS.remove(storage, packet.sequence);
    Some(callback_msg(
        request,
        Callback::Error("timeout".to_string()),
    ))
}

#[cw_serde]
struct PendingCallback {
    initiator: Addr,
    initiator_msg: Binary,
    receiver: Addr,
}

const CALLBACKS: Map<u64, PendingCallback> = Map::new("polytone-callbacks");
const SEQ: Item<u64> = Item::new("polytone-ibc-seq");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let c = Callback::Success(vec![None]);
        assert_eq!(
            to_binary(&c).unwrap().to_string(),
            // base64 of `{"success":[null]}`
            "eyJzdWNjZXNzIjpbbnVsbF19"
        )
    }
}
