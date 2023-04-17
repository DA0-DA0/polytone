use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// (Connection-ID, Remote port) of this contract's pair.
pub const CONNECTION_REMOTE_PORT: Item<(String, String)> = Item::new("a");

/// Channel-ID of the channel currently connected. Holds no value when
/// no channel is active.
pub const CHANNEL: Item<String> = Item::new("b");

/// The controller of this note. See documentation for the
/// `controller` field on `InstantiateMsg` for more information.
pub const CONTROLLER: Item<Addr> = Item::new("c");
