use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const WTOKEN_CONTRACT: Item<Addr> = Item::new("wtoken_contract");
pub const USDSIM_CONTRACT: Item<Addr> = Item::new("usdsim_contract");
pub const WTOKEN_BALANCES: Map<Addr, WtokenBalance> = Map::new("wtoken_balances");
pub const WTOEKN_TOTAL_BALANCE: Item<WtokenBalance> = Item::new("wtoken_total_balance");
pub const REMAINING_USDSIM: Item<UsdsimBalance> = Item::new("remaining_usdsim");

#[cw_serde]
pub struct WtokenBalance(pub Uint128);

#[cw_serde]
pub struct UsdsimBalance(pub Uint128);

impl From<u128> for WtokenBalance {
    fn from(value: u128) -> Self {
        Self(Uint128::from(value))
    }
}

impl From<u128> for UsdsimBalance {
    fn from(value: u128) -> Self {
        Self(Uint128::from(value))
    }
}
