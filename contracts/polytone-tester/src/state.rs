use cw_storage_plus::Item;
use polytone::callback::CallbackMessage;

pub(crate) const CALLBACK_HISTORY: Item<Vec<CallbackMessage>> = Item::new("a");
pub(crate) const HELLO_CALL_HISTORY: Item<Vec<String>> = Item::new("b");
