use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{REMAINING_USDSIM, WTOEKN_TOTAL_BALANCE, WTOKEN_CONTRACT},
};

use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage,
};

const USDSIM_DENOM: &str = "usdsim";

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let InstantiateMsg { wtoken_contract } = msg;
    let store = deps.storage;
    let wtoken_addr = deps.api.addr_validate(&wtoken_contract)?;

    WTOKEN_CONTRACT.save(store, &wtoken_addr)?;
    WTOEKN_TOTAL_BALANCE.save(store, &0u128.into())?;
    REMAINING_USDSIM.save(store, &0u128.into())?;

    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    Ok(match msg {
        WtokenBalance { address } => to_json_binary(&query::wtoken_balance(deps, address)?)?,
        WtokenTotalBalance {} => to_json_binary(&query::wtoken_total_balance(deps)?)?,
        RemainingUsdsim {} => to_json_binary(&query::remaining_usdsim(deps)?)?,
    })
}

mod query {
    use crate::state::{UsdsimBalance, WtokenBalance, WTOKEN_BALANCES};

    use super::*;

    pub fn wtoken_balance(deps: Deps, address: String) -> StdResult<WtokenBalance> {
        let addr = deps.api.addr_validate(&address)?;
        WTOKEN_BALANCES.load(deps.storage, addr)
    }

    pub fn wtoken_total_balance(deps: Deps) -> StdResult<WtokenBalance> {
        WTOEKN_TOTAL_BALANCE.load(deps.storage)
    }

    pub fn remaining_usdsim(deps: Deps) -> StdResult<UsdsimBalance> {
        REMAINING_USDSIM.load(deps.storage)
    }
}

#[allow(dead_code)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        Receive(receive_msg) => exec::receive_wtoken(deps, info, receive_msg),
        ReceiveUsdsim {} => exec::receive_usdsim(deps, info),
    }
}

mod exec {
    use cosmwasm_std::{coins, BankMsg, Order, Uint128};
    use cw20::Cw20ReceiveMsg;

    use crate::state::{
        UsdsimBalance, WtokenBalance, REMAINING_USDSIM, WTOEKN_TOTAL_BALANCE, WTOKEN_BALANCES,
    };

    use super::*;

    pub fn receive_wtoken(
        deps: DepsMut,
        info: MessageInfo,
        msg: Cw20ReceiveMsg,
    ) -> Result<Response, ContractError> {
        let wtoken_contract = WTOKEN_CONTRACT.load(deps.storage)?;
        let sender = deps.api.addr_validate(&msg.sender)?;

        if info.sender == wtoken_contract {
            receive_wtoken_inner(deps.storage, sender, msg.amount)
        } else {
            Err(ContractError::InvalidWtokenAddress {
                address: info.sender,
            })
        }
    }

    fn receive_wtoken_inner(
        storage: &mut dyn Storage,
        sender: Addr,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        if amount == Uint128::zero() {
            return Err(ContractError::ZeroAmountReceived);
        }

        WTOKEN_BALANCES.update(storage, sender, |may_balance| {
            let new_amount = match may_balance {
                None => amount,
                Some(WtokenBalance(prev_amount)) => amount + prev_amount,
            };
            Result::<_, ContractError>::Ok(WtokenBalance(new_amount))
        })?;

        WTOEKN_TOTAL_BALANCE.update(storage, |WtokenBalance(prev_amount)| {
            Result::<_, ContractError>::Ok(WtokenBalance(prev_amount + amount))
        })?;

        Ok(Response::new())
    }

    // We assume receiving USDsim every 24 hours
    pub fn receive_usdsim(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
        let sent_usdsim = info.funds.iter().find(|coin| coin.denom == USDSIM_DENOM);

        if let Some(coin) = sent_usdsim {
            receive_usdsim_inner(deps.storage, coin.amount)
        } else {
            Err(ContractError::NoUsdsimTokensSent)
        }
    }

    fn receive_usdsim_inner(
        storage: &mut dyn Storage,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        if amount == Uint128::zero() {
            return Ok(Response::new());
        }

        let total_balance = WTOEKN_TOTAL_BALANCE.load(storage)?;
        let remaining_usdsim = REMAINING_USDSIM.load(storage)?;

        let (resp, remaining_amount) = WTOKEN_BALANCES
            .range(storage, None, None, Order::Ascending)
            .try_fold(
                (Response::new(), amount + remaining_usdsim.0),
                |(resp, remaining_amount), item| -> Result<_, ContractError> {
                    let (recipient_addr, balance) = item?;
                    let reward_amount = balance.0.checked_mul(amount)? / total_balance.0;
                    let resp = resp.add_message(BankMsg::Send {
                        to_address: recipient_addr.into(),
                        amount: coins(reward_amount.into(), USDSIM_DENOM),
                    });
                    Ok((resp, remaining_amount - reward_amount))
                },
            )?;

        REMAINING_USDSIM.save(storage, &UsdsimBalance(remaining_amount))?;

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        coins, from_json,
        testing::{mock_dependencies_with_balance, mock_env, mock_info, MockApi},
        Addr,
    };
    use cw20::Cw20ReceiveMsg;
    use lazy_static::lazy_static;
    use serde::de::DeserializeOwned;

    use crate::{
        execute, query,
        state::{UsdsimBalance, WtokenBalance},
    };

    use super::*;

    pub fn query2<T>(deps: Deps, msg: QueryMsg) -> StdResult<T>
    where
        T: DeserializeOwned,
    {
        let env = mock_env();
        let data = query(deps, env, msg)?;
        from_json(data)
    }

    lazy_static! {
        static ref WTOKEN_ADDRESS: String = {
            let mock_api = MockApi::default();
            mock_api.addr_make("wtoken-address").to_string()
        };
    }

    fn do_instantiation(mut deps: DepsMut) {
        let instantiate_msg = InstantiateMsg {
            wtoken_contract: WTOKEN_ADDRESS.clone(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    fn execute_receive_wtoken(
        deps: DepsMut,
        sender: &str,
        amount: u128,
    ) -> Result<Response, ContractError> {
        let env = mock_env();
        let info = mock_info(&WTOKEN_ADDRESS, &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: sender.into(),
            amount: amount.into(),
            msg: Binary::default(),
        });
        execute(deps, env.clone(), info, msg)
    }

    fn execute_receive_usdsim(
        deps: DepsMut,
        sender: &str,
        amount: u128,
    ) -> Result<Response, ContractError> {
        let env = mock_env();
        let info = mock_info(sender, &coins(amount, USDSIM_DENOM));
        let msg = ExecuteMsg::ReceiveUsdsim {};
        execute(deps, env.clone(), info, msg)
    }

    #[test]
    fn instantiation() {
        let mut deps = mock_dependencies_with_balance(&[]);

        let instantiate_msg = InstantiateMsg {
            wtoken_contract: WTOKEN_ADDRESS.clone(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = QueryMsg::WtokenTotalBalance {};
        let total_balance: WtokenBalance = query2(deps.as_ref(), msg).unwrap();
        assert_eq!(total_balance, WtokenBalance::from(0u128));

        let msg = QueryMsg::RemainingUsdsim {};
        let remaining_usdsim: WtokenBalance = query2(deps.as_ref(), msg).unwrap();
        assert_eq!(remaining_usdsim, WtokenBalance::from(0u128));
    }

    #[test]
    fn exec_receive_wtoken() {
        let mut deps = mock_dependencies_with_balance(&[]);
        do_instantiation(deps.as_mut());

        let sender = deps.api.addr_make("user0001").to_string();
        let sender2 = deps.api.addr_make("user0002").to_string();

        // --- execution 1 ---

        execute_receive_wtoken(deps.as_mut(), &sender, 100u128).unwrap();

        let msg = QueryMsg::WtokenBalance {
            address: sender.clone(),
        };
        let balance_query_resp: WtokenBalance = query2(deps.as_ref(), msg).unwrap();

        let msg = QueryMsg::WtokenTotalBalance {};
        let total_balance_query_resp: WtokenBalance = query2(deps.as_ref(), msg).unwrap();

        assert_eq!(balance_query_resp, WtokenBalance::from(100u128));
        assert_eq!(total_balance_query_resp, WtokenBalance::from(100u128));

        // --- execution 2 ---

        execute_receive_wtoken(deps.as_mut(), &sender2, 20u128).unwrap();

        let msg = QueryMsg::WtokenBalance {
            address: sender2.clone(),
        };
        let balance_query_resp2: WtokenBalance = query2(deps.as_ref(), msg).unwrap();

        let msg = QueryMsg::WtokenTotalBalance {};
        let total_balance_query_resp2: WtokenBalance = query2(deps.as_ref(), msg).unwrap();

        assert_eq!(balance_query_resp2, WtokenBalance::from(20u128));
        assert_eq!(total_balance_query_resp2, WtokenBalance::from(120u128));

        // --- execution 3 ---

        execute_receive_wtoken(deps.as_mut(), &sender, 50u128).unwrap();

        let msg = QueryMsg::WtokenBalance {
            address: sender.clone(),
        };
        let balance_query_resp: WtokenBalance = query2(deps.as_ref(), msg).unwrap();

        let msg = QueryMsg::WtokenTotalBalance {};
        let total_balance_query_resp: WtokenBalance = query2(deps.as_ref(), msg).unwrap();

        assert_eq!(balance_query_resp, WtokenBalance::from(150u128));
        assert_eq!(total_balance_query_resp, WtokenBalance::from(170u128));
    }

    #[test]
    fn exec_reward_distribution() {
        let mut deps = mock_dependencies_with_balance(&[]);
        do_instantiation(deps.as_mut());

        let sender1 = deps.api.addr_make("user0001").to_string();
        let sender2 = deps.api.addr_make("user0002").to_string();
        let sender3 = deps.api.addr_make("user0003").to_string();
        let usdsim_sender = deps.api.addr_make("user0004").to_string();

        // receive wtoken 1
        let res = execute_receive_wtoken(deps.as_mut(), &sender1, 10u128).unwrap();
        assert_eq!(0, res.messages.len());

        // receive wtoken 2
        let res = execute_receive_wtoken(deps.as_mut(), &sender2, 90u128).unwrap();
        assert_eq!(0, res.messages.len());

        // receive usdsim, day 1
        let res = execute_receive_usdsim(deps.as_mut(), &usdsim_sender, 1000u128).unwrap();
        assert_eq!(2, res.messages.len()); // TODO: add insta.rs for snapshot testing

        let remaining_usdsim_resp: UsdsimBalance =
            query2(deps.as_ref(), QueryMsg::RemainingUsdsim {}).unwrap();

        assert_eq!(remaining_usdsim_resp, UsdsimBalance::from(0u128));

        // receive wtoken 3
        let res = execute_receive_wtoken(deps.as_mut(), &sender3, 100u128).unwrap();
        assert_eq!(0, res.messages.len());

        // receive usdsim, day 2
        let res = execute_receive_usdsim(deps.as_mut(), &usdsim_sender, 1001u128).unwrap();
        assert_eq!(3, res.messages.len()); // TODO: add insta.rs for snapshot testing

        let remaining_usdsim_resp: UsdsimBalance =
            query2(deps.as_ref(), QueryMsg::RemainingUsdsim {}).unwrap();

        assert_eq!(remaining_usdsim_resp, UsdsimBalance::from(1u128));
    }

    #[test]
    fn exec_receive_fail() {
        let mut deps = mock_dependencies_with_balance(&[]);
        do_instantiation(deps.as_mut());

        let sender = deps.api.addr_make("user0001").to_string();
        let not_a_wtoken_nor_usdsim_addr = deps.api.addr_make("xxx").to_string();

        let env = mock_env();
        let info = mock_info(&not_a_wtoken_nor_usdsim_addr, &[]);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: sender.clone(),
            amount: 100u128.into(),
            msg: Binary::default(),
        });
        let err = execute(deps.as_mut(), env.clone(), info, msg).unwrap_err();

        assert_eq!(
            err,
            ContractError::InvalidWtokenAddress {
                address: Addr::unchecked(not_a_wtoken_nor_usdsim_addr)
            },
        );

        let err = execute_receive_wtoken(deps.as_mut(), &sender, 0u128).unwrap_err();

        assert_eq!(err, ContractError::ZeroAmountReceived);
    }
}
