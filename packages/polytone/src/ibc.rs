use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, Empty, IbcChannel, IbcOrder, QueryRequest};
use thiserror::Error;

pub const VERSION: &str = "polytone";

#[cw_serde]
pub struct Packet {
    /// Message sender on the note chain.
    pub sender: String,
    /// Message to execute on voice chain.
    pub msg: Msg,
}

#[cw_serde]
pub enum Msg {
    /// Performs the requested queries on the voice chain and returns a
    /// callback of Vec<QuerierResult>, or ACK-FAIL if unmarshalling
    /// any of the query requests fails.
    Query { msgs: Vec<QueryRequest<Empty>> },
    /// Executes the requested messages on the voice chain on behalf of
    /// the note chain sender. Message receivers can return data
    /// in their callbacks by calling `set_data` on their `Response`
    /// object. Returns a callback of `Vec<Callback>` where index `i`
    /// corresponds to the callback for `msgs[i]`.
    Execute { msgs: Vec<CosmosMsg<Empty>> },
}

#[derive(Error, Debug)]
pub enum OrderVersionError {
    #[error("channel must be unordered")]
    UnUnordered,

    #[error("version missmatch, got ({actual}), expected ({expected})")]
    VersionMissmatch { actual: String, expected: String },
}

pub fn validate_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), OrderVersionError> {
    if channel.order != IbcOrder::Unordered {
        return Err(OrderVersionError::UnUnordered);
    }

    if channel.version != VERSION {
        return Err(OrderVersionError::VersionMissmatch {
            actual: channel.version.to_string(),
            expected: VERSION.to_string(),
        });
    }

    // Make sure that we're talking with a counterparty who speaks the
    // same language as us.
    //
    // For a connection between chain A and chain B being established
    // by chain A, chain B knows counterparty information during
    // `OpenTry` and chain A knows counterparty information during
    // `OpenAck`.
    //
    // During `OpenAck`, chain A's `ibc_channel_open` method is
    // called. At this point, no message has been received from the
    // counterparty, so `counterparty_version` is `None`. The job of
    // the initiator is to check that `channel.version` set by the
    // caller matches the contract's version.
    //
    // For the remainder of the handshake,
    // `counterparty_version.is_some()`.
    if counterparty_version.map_or(true, |version| version == VERSION) {
        Ok(())
    } else {
        Err(OrderVersionError::VersionMissmatch {
            actual: counterparty_version.unwrap().to_string(),
            expected: VERSION.to_string(),
        })
    }
}
