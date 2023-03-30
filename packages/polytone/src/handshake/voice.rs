use cosmwasm_std::{
    from_binary, Binary, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse,
};

use super::{error::HandshakeError, note_version, voice_version};

/// Performs the open step of the IBC handshake for a voice module.
///
/// # Arguments
///
/// - `extensions` the Polytone extensions supported by the caller.
///   Extensions are explained in detail in the polytone spec.
/// - `msg` the message received to open the channel.
pub fn open(
    msg: IbcChannelOpenMsg,
    extensions: &[&str],
) -> Result<IbcChannelOpenResponse, HandshakeError> {
    super::open(msg, extensions, voice_version(), note_version())
}

/// Performs the connect step of the IBC handshake for a voice module.
///
/// # Arguments
///
/// - `extensions` the Polytone extensions supported by the caller.
///   Extensions are explained in detail in the polytone spec.
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
