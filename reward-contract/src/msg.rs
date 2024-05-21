use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::Cw20ReceiveMsg;

use crate::state::{UsdsimBalance, WtokenBalance};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(WtokenBalance)]
    WtokenBalance { address: String },
    #[returns(WtokenBalance)]
    WtokenTotalBalance {},
    #[returns(UsdsimBalance)]
    RemainingUsdsim {},
}

#[cw_serde]
pub struct InstantiateMsg {
    pub wtoken_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    ReceiveUsdsim {},
}
