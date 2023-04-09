use cosmwasm_schema::cw_serde;
use cosmwasm_std::{from_binary, to_binary, Binary, IbcAcknowledgement, SubMsgResponse};

pub use crate::callback::Callback;
use crate::callback::{CallbackData, RequestType};

pub type Ack = Callback;

#[cw_serde]
pub struct InterlError(String);

/// Serializes an ACK-SUCCESS containing the provided data.
pub fn ack_success_query(c: Vec<Binary>) -> Binary {
    to_binary(&Callback::Query(CallbackData::Success(c))).unwrap()
}

/// Serializes an ACK-SUCCESS containing the provided data.
pub fn ack_success_execute(c: Vec<SubMsgResponse>) -> Binary {
    to_binary(&Callback::Execute(CallbackData::Success(c))).unwrap()
}

/// Serializes an ACK-FAIL containing the provided error.
pub fn ack_fail(err: String) -> Binary {
    to_binary(&InterlError(err)).unwrap()
}

/// Unmarshals an ACK from an acknowledgement returned by the SDK. If
/// the returned acknowledgement can not be parsed into an ACK,
/// err(base64(ack)) is returned.
///
/// # Note
///
/// Occasionally you will receive ACKs from the SDK, and not
/// your counterparty contract. I do not know all cases this will
/// occur, but I do know it happens if a field on the packet data is
/// set to an empty string. That being the case, the SDK will return
/// an error in the form:
///
/// ```json
/// {"error":"Empty attribute value. Key: <key w/ empty string>: invalid event"}
/// ```
///
/// This means that even if you know all of the error types returned
/// by your counterparty contract, unless you know all the error types
/// the SDK will throw, you can't assume error strings will be regular
/// for unmarshaled ACKs.
///
/// For an example of this, see this integration test:
///
/// <https://github.com/public-awesome/ics721/blob/3af19e421a95aec5291a0cabbe796c58698ac97f/e2e/adversarial_test.go#L274-L285>
pub fn unmarshal_ack(ack: &IbcAcknowledgement, request_type: RequestType) -> Ack {
    if let Ok(err) = from_binary::<InterlError>(&ack.data) {
        return match request_type {
            RequestType::Execute => Callback::Execute(CallbackData::Error(err.0)),
            RequestType::Query => Callback::Query(CallbackData::Error(err.0)),
        };
    }
    from_binary(&ack.data).unwrap_or_else(|_| match request_type {
        RequestType::Execute => Callback::Execute(CallbackData::Error(ack.data.to_base64())),
        RequestType::Query => Callback::Query(CallbackData::Error(ack.data.to_base64())),
    })
}
