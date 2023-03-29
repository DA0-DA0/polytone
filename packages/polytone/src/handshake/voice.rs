use cosmwasm_std::{
    from_binary, Binary, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse,
};

use super::{error::HandshakeError, voice_version};

pub fn open(
    msg: IbcChannelOpenMsg,
    extensions: &[&str],
) -> Result<IbcChannelOpenResponse, HandshakeError> {
    super::open(msg, extensions, voice_version())
}

pub fn connect(msg: IbcChannelConnectMsg, extensions: &[&str]) -> Result<(), HandshakeError> {
    match msg {
        IbcChannelConnectMsg::OpenAck {
            channel: _,
            counterparty_version,
        } => {
            let proposed_version: Vec<String> =
                from_binary(&Binary::from_base64(&counterparty_version).unwrap()).unwrap();
            let subseteq_violation = proposed_version
                .iter()
                .find(|e| !extensions.contains(&e.as_str()));
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
