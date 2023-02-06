use cosmwasm_std::{Addr, Binary};
use cw_storage_plus::Item;

/// Stores the instantiator of the contract.
pub const INSTANTIATOR: Item<Addr> = Item::new("owner");

/// Stores a list of callback's currently being collected. Has no
/// value if none are being collected.
pub const COLLECTOR: Item<Vec<Option<Binary>>> = Item::new("callbacks");
