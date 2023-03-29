use cosmwasm_std::{
    from_binary, Binary, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse,
};

use super::{error::HandshakeError, note_version};

pub fn open(
    msg: IbcChannelOpenMsg,
    extensions: &[&str],
) -> Result<IbcChannelOpenResponse, HandshakeError> {
    super::open(msg, extensions, note_version())
}

pub fn connect(msg: IbcChannelConnectMsg, extensions: &[&str]) -> Result<(), HandshakeError> {
    match msg {
        IbcChannelConnectMsg::OpenAck {
            channel: _,
            counterparty_version,
        } => {
            let proposed_version: Vec<String> =
                from_binary(&Binary::from_base64(&counterparty_version).unwrap()).unwrap();
            let subseteq_violation = extensions
                .iter()
                .find(|e| !proposed_version.contains(&e.to_string()));
            match subseteq_violation {
                None => Ok(()),
                Some(first_voilation) => {
                    Err(HandshakeError::Unspeakable(first_voilation.to_string()))
                }
            }
        }
        IbcChannelConnectMsg::OpenConfirm { channel: _ } => Ok(()),
    }
}
