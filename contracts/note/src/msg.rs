use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Empty, QueryRequest, Uint64};

use polytone::callback::CallbackRequest;

#[cw_serde]
pub struct InstantiateMsg {
    /// This contract pairs with the first voice module that a relayer
    /// connects it with, or the pair specified here. Once it has a
    /// pair, it will never handshake with a different voice module,
    /// even after channel closure. This ensures that there will only
    /// ever be one voice for every note.
    pub pair: Option<Pair>,

    /// This is the controller of the note. If a controller is set:
    ///
    /// 1. Only the controller may execute messages.
    /// 2. The controller mat execute messages on behalf of any
    ///    address.
    ///
    /// The controller allows sequencing of IBC-transfers and actions
    /// on the counterparty chain. For example, the controller of a
    /// note could:
    ///
    /// 1. Receive tokens from initiator.
    /// 2. Transfer tokens to initators remote account.
    /// 3. Perform an action on the initiator's behalf on the remote
    ///    chain.
    ///
    /// For more discussion of the controller, see "How Polytone
    /// Supports Outposts":
    ///
    /// <https://github.com/DA0-DA0/polytone/wiki/How-Polytone-Supports-Outposts>
    pub controller: Option<String>,

    pub block_max_gas: Uint64,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Performs the requested queries on the voice chain and returns
    /// a callback of Vec<QuerierResult>, or ACK-FAIL if unmarshalling
    /// any of the query requests fails.
    Query {
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
        msgs: Vec<CosmosMsg<Empty>>,
        callback: Option<CallbackRequest>,
        timeout_seconds: Uint64,

        /// Must be `None` if no controller is set, or `Some(address)`
        /// if a controller is set.
        on_behalf_of: Option<String>,
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
    /// This note's controller, or None.
    #[returns(Option<String>)]
    Controller,
    /// Returns the remote address for the provided local address. If
    /// no account exists, returns `None`. An account can be created
    /// by calling `ExecuteMsg::Execute` with the sender being
    /// `local_address`.
    #[returns(Option<String>)]
    RemoteAddress { local_address: String },
    /// Currently set gas limit
    #[returns(Uint64)]
    BlockMaxGas,
}

/// This contract's voice. There is one voice per note, and many notes
/// per voice.
#[cw_serde]
pub struct Pair {
    pub connection_id: String,
    pub remote_port: String,
}

#[cw_serde]
pub enum MigrateMsg {
    WithUpdate {
        block_max_gas: Uint64,
    }
}