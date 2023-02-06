use cw_storage_plus::Item;
use polytone::callback::CallbackMessage;

pub(crate) const CALLBACK_HISTORY: Item<Vec<CallbackMessage>> = Item::new("a");
