use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use thiserror::Error;

// Take care when adding variants to this type that an attacker can't
// create an error that will deserailize into a base64-encoded
// `ExecutionFailure`, as the string representation of
// `ExecutionFailure` is a base64-encoded, JSON `ExecutionFailure`.

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Parse(#[from] ParseReplyError),

    #[error("caller must be the contract instantiator")]
    NotInstantiator,

    #[error("executing message {index}: {error}")]
    MsgError { index: u64, error: String },
}
