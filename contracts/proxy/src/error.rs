use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Reply(#[from] ParseReplyError),

    #[error("caller must be the contract instantiator")]
    NotInstantiator,

    #[error("host chain, msg ({idx}), error ({error})")]
    ExecutionFailure { idx: u64, error: String },
}
