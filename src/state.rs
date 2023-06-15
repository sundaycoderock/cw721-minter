use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[cw_serde]
pub struct State {
    pub admin: Addr,
    pub nft_contract: Addr,
}

pub const STATE: Item<State> = Item::new("state");
