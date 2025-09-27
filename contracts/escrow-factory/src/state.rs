use cosmwasm_std::HexBinary;
use cw_storage_plus::Item;
pub const ESCROW_DST_CODE_ID: Item<u64> = Item::new("escrow_dst_code_id");
pub const ESCROW_DST_CODE_HASH: Item<HexBinary> = Item::new("escrow_dst_code_hash");
pub const SAFETY_DEPOSIT_TOKEN: Item<String> = Item::new("safety_deposit_token");

pub const RESCUE_DELAY: u32 = 86400;