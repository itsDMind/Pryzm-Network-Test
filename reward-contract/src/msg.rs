use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_json_binary, CosmosMsg, StdResult, Uint128, WasmMsg};
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
    pub usdsim_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub struct TransferUsdsimMsg {
    pub recipient: String,
    pub amount: Uint128,
}

impl TransferUsdsimMsg {
    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&self)?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}
