use cw_storage_plus::Item;
pub const IMMUTABLE_HASH: Item<String> = Item::new("immutable_hash");
pub const SAFETY_DEPOSIT_TOKEN: Item<String> = Item::new("safety_deposit_denom");
pub const RESCUE_DELAY: Item<u32> = Item::new("rescue_delay");
