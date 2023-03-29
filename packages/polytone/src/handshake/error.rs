use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("protocol missmatch, got {actual}, expected {expected}")]
    ProtocolMissmatch { actual: String, expected: String },
    #[error("channel must be unordered")]
    UnUnordered,
    #[error("only a note and voice may connect")]
    WrongCounterparty,
    #[error("note can say ({0}), but voice can not speak it")]
    Unspeakable(String),
}
