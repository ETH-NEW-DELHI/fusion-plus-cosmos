use cosmwasm_std::HexBinary;
use cw_storage_plus::Item;
pub const FACTORY_CONTRACT_ADDRESS: Item<String> = Item::new("factory_contract_address");
pub const ESCROW_DST_CODE_HASH: Item<HexBinary> = Item::new("escrow_dst_code_hash");
pub const SAFETY_DEPOSIT_TOKEN: Item<String> = Item::new("safety_deposit_denom");
pub const RESCUE_DELAY: Item<u32> = Item::new("rescue_delay");
