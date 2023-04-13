use cosmwasm_std::{from_binary, to_binary, Binary, IbcAcknowledgement, SubMsgResponse, Uint64};

pub use crate::callback::Callback;
use crate::callback::{ErrorResponse, ExecutionResponse};

/// wasmd 0.32+ will not return a hardcoded ICS-20 ACK if
/// ibc_packet_receive errors [1] so we can safely use an ACK format
/// that is not ICS-20 error-variant compatible.
///
/// [1]: https://github.com/CosmWasm/wasmd/issues/1305#issuecomment-1489871618
pub type Ack = Callback;

/// Serializes an ACK-SUCCESS containing the provided data.
pub fn ack_query_success(result: Vec<Binary>) -> Binary {
    to_binary(&Callback::Query(Ok(result))).unwrap()
}

/// Serializes an ACK-SUCCESS for a query that failed.
pub fn ack_query_fail(message_index: Uint64, error: String) -> Binary {
    to_binary(&Callback::Query(Err(ErrorResponse {
        message_index,
        error,
    })))
    .unwrap()
}

/// Serializes an ACK-SUCCESS for execution that succeded.
pub fn ack_execute_success(result: Vec<SubMsgResponse>, executed_by: String) -> Binary {
    to_binary(&Callback::Execute(Ok(ExecutionResponse {
        result,
        executed_by,
    })))
    .unwrap()
}

/// Serializes an ACK-SUCCESS for execution that failed.
pub fn ack_execute_fail(message_index: Uint64, error: String) -> Binary {
    to_binary(&Callback::Execute(Err(ErrorResponse {
        message_index,
        error,
    })))
    .unwrap()
}

/// Serializes an ACK-FAIL containing the provided error.
pub fn ack_fail(err: String) -> Binary {
    to_binary(&Callback::InternalError(err)).unwrap()
}

/// Unmarshals an ACK from an acknowledgement returned by the SDK. If
/// the returned acknowledgement can not be parsed into an ACK,
/// err(base64(ack)) is returned.
pub fn unmarshal_ack(ack: &IbcAcknowledgement) -> Ack {
    from_binary(&ack.data).unwrap_or_else(|e| {
        Callback::InternalError(format!(
            "error unmarshaling ack ({}): {}",
            ack.data.to_base64(),
            e
        ))
    })
}
