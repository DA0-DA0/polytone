use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Empty, QueryRequest, Uint64};

use polytone::callback::CallbackRequest;

#[cw_serde]
pub struct InstantiateMsg {
    /// This contract pairs with the first voice module that a relayer
    /// connects it with, or the pair specified here. Once it has a
    /// pair, it will never handshake with a different voice module,
    /// even after channel closure.
    pub pair: Option<Pair>,
    /// This is the controller of the note. if controller is set
    /// only that address can execute msgs on this note.
    pub controller: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Performs the requested queries on the voice chain and returns
    /// a callback of Vec<QuerierResult>, or ACK-FAIL if unmarshalling
    /// any of the query requests fails.
    Query {
        on_behalf_of: Option<String>,
        msgs: Vec<QueryRequest<Empty>>,
        callback: CallbackRequest,
        timeout_seconds: Uint64,
    },
    /// Executes the requested messages on the voice chain on behalf
    /// of the note chain sender. Message receivers can return data in
    /// their callbacks by calling `set_data` on their `Response`
    /// object. Optionaly, returns a callback of `Vec<Callback>` where
    /// index `i` corresponds to the callback for `msgs[i]`.
    Execute {
        on_behalf_of: Option<String>,
        msgs: Vec<CosmosMsg<Empty>>,
        callback: Option<CallbackRequest>,
        timeout_seconds: Uint64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// This channel this note is currently connected to, or none if
    /// no channel is connected.
    #[returns(Option<String>)]
    ActiveChannel,
    /// The contract's corresponding voice on a remote chain.
    #[returns(Option<Pair>)]
    Pair,
}

/// This contract's voice. There is one voice per note, and many notes
/// per voice.
#[cw_serde]
pub struct Pair {
    pub connection_id: String,
    pub remote_port: String,
}
