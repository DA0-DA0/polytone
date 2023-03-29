use cosmwasm_std::{
    to_binary, Ibc3ChannelOpenResponse, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcOrder,
};

use error::HandshakeError;

pub const POLYTONE_VERSION: &str = "polytone-1";

pub fn note_version() -> String {
    format!("{}-note", POLYTONE_VERSION)
}

pub fn voice_version() -> String {
    format!("{}-voice", POLYTONE_VERSION)
}

fn open(
    msg: IbcChannelOpenMsg,
    extensions: &[&str],
    version: String,
) -> Result<IbcChannelOpenResponse, HandshakeError> {
    match msg {
        IbcChannelOpenMsg::OpenInit { channel } => {
            if channel.version != POLYTONE_VERSION {
                Err(HandshakeError::ProtocolMissmatch {
                    actual: channel.version,
                    expected: POLYTONE_VERSION.to_string(),
                })
            } else if channel.order != IbcOrder::Unordered {
                Err(HandshakeError::UnUnordered)
            } else {
                Ok(Some(Ibc3ChannelOpenResponse { version }))
            }
        }
        IbcChannelOpenMsg::OpenTry {
            channel,
            counterparty_version,
        } => {
            if counterparty_version != voice_version() {
                Err(HandshakeError::WrongCounterparty)
            } else if channel.order != IbcOrder::Unordered {
                Err(HandshakeError::UnUnordered)
            } else {
                Ok(Some(Ibc3ChannelOpenResponse {
                    version: to_binary(extensions).unwrap().to_base64(),
                }))
            }
        }
    }
}

pub mod error;
pub mod note;
pub mod voice;
