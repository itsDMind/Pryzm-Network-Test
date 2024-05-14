#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, execute_update_minter,
    query_balance, query_minter, query_token_info,
};

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
};

use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};

const TOKEN_NAME: &str = "Wtoken";
const TOKEN_SYMBOL: &str = "WTK";
const TOKEN_DECIMALS: u8 = 6;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let InstantiateMsg {
        initial_balances,
        mint,
    } = msg;

    let cw20_init_msg = cw20_base::msg::InstantiateMsg {
        name: TOKEN_NAME.into(),
        symbol: TOKEN_SYMBOL.into(),
        initial_balances,
        decimals: TOKEN_DECIMALS,
        mint,
        marketing: None,
    };

    cw20_base::contract::instantiate(deps, env, info, cw20_init_msg)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;

    Ok(match msg {
        TokenInfo {} => to_json_binary(&query_token_info(deps)?)?,
        Balance { address } => to_json_binary(&query_balance(deps, address)?)?,
        Minter {} => to_json_binary(&query_minter(deps)?)?,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    Ok(match msg {
        Transfer { recipient, amount } => execute_transfer(deps, env, info, recipient, amount)?,
        Burn { amount } => execute_burn(deps, env, info, amount)?,
        Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg)?,
        Mint { recipient, amount } => execute_mint(deps, env, info, recipient, amount)?,
        UpdateMinter { new_minter } => execute_update_minter(deps, env, info, new_minter)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::error::CW20BaseError;
    use cosmwasm_std::{
        coins, from_json,
        testing::{mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info},
        CosmosMsg, StdError, StdResult, SubMsg, Uint128, WasmMsg,
    };
    use cw20::{BalanceResponse, Cw20Coin, Cw20ReceiveMsg, MinterResponse, TokenInfoResponse};

    use super::*;

    fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
        query_balance(deps, address.into()).unwrap().balance
    }

    pub fn query_minter(deps: Deps) -> StdResult<Option<MinterResponse>> {
        let meta = cw20_base::state::TOKEN_INFO.load(deps.storage)?;
        let minter = match meta.mint {
            Some(m) => Some(MinterResponse {
                minter: m.minter.into(),
                cap: m.cap,
            }),
            None => None,
        };
        Ok(minter)
    }

    // this will set up the instantiation for other tests
    fn do_instantiate(deps: DepsMut, addr: &str, amount: Uint128) -> TokenInfoResponse {
        _do_instantiate(deps, addr, amount, None)
    }

    fn do_instantiate_with_minter(
        deps: DepsMut,
        addr: &str,
        amount: Uint128,
        minter: &str,
        cap: Option<Uint128>,
    ) -> TokenInfoResponse {
        _do_instantiate(
            deps,
            addr,
            amount,
            Some(MinterResponse {
                minter: minter.to_string(),
                cap,
            }),
        )
    }

    // this will set up the instantiation for other tests
    fn _do_instantiate(
        mut deps: DepsMut,
        addr: &str,
        amount: Uint128,
        mint: Option<MinterResponse>,
    ) -> TokenInfoResponse {
        let instantiate_msg = InstantiateMsg {
            initial_balances: vec![Cw20Coin {
                address: addr.to_string(),
                amount,
            }],
            mint: mint.clone(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let meta = query_token_info(deps.as_ref()).unwrap();
        assert_eq!(
            meta,
            TokenInfoResponse {
                name: TOKEN_NAME.into(),
                symbol: TOKEN_SYMBOL.into(),
                decimals: TOKEN_DECIMALS,
                total_supply: amount,
            }
        );
        assert_eq!(get_balance(deps.as_ref(), addr), amount);
        assert_eq!(query_minter(deps.as_ref()).unwrap(), mint,);
        meta
    }

    macro_rules! cw20_base_std_err {
        ($e:pat) => {
            ContractError::CW20Base(CW20BaseError::Std($e))
        };
    }

    #[test]
    fn instantiate_multiple_accounts() {
        let mut deps = mock_dependencies();
        let amount1 = Uint128::from(11223344u128);
        let addr1 = deps.api.addr_make("addr0001").to_string();
        let amount2 = Uint128::from(7890987u128);
        let addr2 = deps.api.addr_make("addr0002").to_string();
        let info = mock_info("creator", &[]);
        let env = mock_env();

        // Fails with duplicate addresses
        let instantiate_msg = InstantiateMsg {
            initial_balances: vec![
                Cw20Coin {
                    address: addr1.clone(),
                    amount: amount1,
                },
                Cw20Coin {
                    address: addr1.clone(),
                    amount: amount2,
                },
            ],
            mint: None,
        };
        let err =
            instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::CW20Base(cw20_base::ContractError::DuplicateInitialBalanceAddresses {})
        );

        // Works with unique addresses
        let instantiate_msg = InstantiateMsg {
            initial_balances: vec![
                Cw20Coin {
                    address: addr1.clone(),
                    amount: amount1,
                },
                Cw20Coin {
                    address: addr2.clone(),
                    amount: amount2,
                },
            ],
            mint: None,
        };
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(
            query_token_info(deps.as_ref()).unwrap(),
            TokenInfoResponse {
                name: TOKEN_NAME.into(),
                symbol: TOKEN_SYMBOL.into(),
                decimals: 6,
                total_supply: amount1 + amount2,
            }
        );
        assert_eq!(get_balance(deps.as_ref(), addr1), amount1);
        assert_eq!(get_balance(deps.as_ref(), addr2), amount2);
    }

    #[test]
    fn queries_work() {
        let mut deps = mock_dependencies_with_balance(&coins(2, TOKEN_NAME));

        let addr1 = deps.api.addr_make("addr0001").to_string();
        let addr2 = deps.api.addr_make("addr0002").to_string();

        let amount1 = Uint128::from(12340000u128);

        let expected = do_instantiate(deps.as_mut(), &addr1, amount1);

        // check meta query
        let loaded = query_token_info(deps.as_ref()).unwrap();
        assert_eq!(expected, loaded);

        let _info = mock_info("test", &[]);
        let env = mock_env();
        // check balance query (full)
        let data = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Balance { address: addr1 },
        )
        .unwrap();
        let loaded: BalanceResponse = from_json(data).unwrap();
        assert_eq!(loaded.balance, amount1);

        // check balance query (empty)
        let data = query(deps.as_ref(), env, QueryMsg::Balance { address: addr2 }).unwrap();
        let loaded: BalanceResponse = from_json(data).unwrap();
        assert_eq!(loaded.balance, Uint128::zero());
    }

    #[test]
    fn transfer() {
        let mut deps = mock_dependencies_with_balance(&coins(2, TOKEN_NAME));
        let addr1 = deps.api.addr_make("addr0001").to_string();
        let addr2 = deps.api.addr_make("addr0002").to_string();
        let amount1 = Uint128::from(12340000u128);
        let transfer = Uint128::from(76543u128);
        let too_much = Uint128::from(12340321u128);

        do_instantiate(deps.as_mut(), &addr1, amount1);

        // Allows transferring 0
        let info = mock_info(addr1.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Transfer {
            recipient: addr2.clone(),
            amount: Uint128::zero(),
        };
        execute(deps.as_mut(), env, info, msg).unwrap();

        // cannot send more than we have
        let info = mock_info(addr1.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Transfer {
            recipient: addr2.clone(),
            amount: too_much,
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, cw20_base_std_err!(StdError::Overflow { .. })));

        // cannot send from empty account
        let info = mock_info(addr2.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Transfer {
            recipient: addr1.clone(),
            amount: transfer,
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, cw20_base_std_err!(StdError::Overflow { .. })));

        // valid transfer
        let info = mock_info(addr1.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Transfer {
            recipient: addr2.clone(),
            amount: transfer,
        };
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let remainder = amount1.checked_sub(transfer).unwrap();
        assert_eq!(get_balance(deps.as_ref(), addr1), remainder);
        assert_eq!(get_balance(deps.as_ref(), addr2), transfer);
        assert_eq!(
            query_token_info(deps.as_ref()).unwrap().total_supply,
            amount1
        );
    }

    #[test]
    fn burn() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let addr1 = deps.api.addr_make("addr0001").to_string();
        let amount1 = Uint128::from(12340000u128);
        let burn = Uint128::from(76543u128);
        let too_much = Uint128::from(12340321u128);

        do_instantiate(deps.as_mut(), &addr1, amount1);

        // Allows burning 0
        let info = mock_info(addr1.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Burn {
            amount: Uint128::zero(),
        };
        execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(
            query_token_info(deps.as_ref()).unwrap().total_supply,
            amount1
        );

        // cannot burn more than we have
        let info = mock_info(addr1.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Burn { amount: too_much };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, cw20_base_std_err!(StdError::Overflow { .. })));
        assert_eq!(
            query_token_info(deps.as_ref()).unwrap().total_supply,
            amount1
        );

        // valid burn reduces total supply
        let info = mock_info(addr1.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Burn { amount: burn };
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let remainder = amount1.checked_sub(burn).unwrap();
        assert_eq!(get_balance(deps.as_ref(), addr1), remainder);
        assert_eq!(
            query_token_info(deps.as_ref()).unwrap().total_supply,
            remainder
        );
    }

    #[test]
    fn send() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let addr1 = deps.api.addr_make("addr0001").to_string();
        let contract = deps.api.addr_make("contract0001").to_string();
        let amount1 = Uint128::from(12340000u128);
        let transfer = Uint128::from(76543u128);
        let too_much = Uint128::from(12340321u128);
        let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());

        do_instantiate(deps.as_mut(), &addr1, amount1);

        // Allows sending 0
        let info = mock_info(addr1.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Send {
            contract: contract.clone(),
            amount: Uint128::zero(),
            msg: send_msg.clone(),
        };
        execute(deps.as_mut(), env, info, msg).unwrap();

        // cannot send more than we have
        let info = mock_info(addr1.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Send {
            contract: contract.clone(),
            amount: too_much,
            msg: send_msg.clone(),
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, cw20_base_std_err!(StdError::Overflow { .. })));

        // valid transfer
        let info = mock_info(addr1.as_ref(), &[]);
        let env = mock_env();
        let msg = ExecuteMsg::Send {
            contract: contract.clone(),
            amount: transfer,
            msg: send_msg.clone(),
        };
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.messages.len(), 1);

        // ensure proper send message sent
        // this is the message we want delivered to the other side
        let binary_msg = Cw20ReceiveMsg {
            sender: addr1.clone(),
            amount: transfer,
            msg: send_msg,
        }
        .into_json_binary()
        .unwrap();
        // and this is how it must be wrapped for the vm to process it
        assert_eq!(
            res.messages[0],
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract.clone(),
                msg: binary_msg,
                funds: vec![],
            }))
        );

        // ensure balance is properly transferred
        let remainder = amount1.checked_sub(transfer).unwrap();
        assert_eq!(get_balance(deps.as_ref(), addr1), remainder);
        assert_eq!(get_balance(deps.as_ref(), contract), transfer);
        assert_eq!(
            query_token_info(deps.as_ref()).unwrap().total_supply,
            amount1
        );
    }

    #[test]
    fn can_mint_by_minter() {
        let mut deps = mock_dependencies();

        let genesis = deps.api.addr_make("genesis").to_string();
        let amount = Uint128::new(11223344);
        let minter = deps.api.addr_make("asmodat").to_string();
        let limit = Uint128::new(511223344);
        do_instantiate_with_minter(deps.as_mut(), &genesis, amount, &minter, Some(limit));

        // minter can mint coins to some winner
        let winner = deps.api.addr_make("winner").to_string();
        let prize = Uint128::new(222_222_222);
        let msg = ExecuteMsg::Mint {
            recipient: winner.clone(),
            amount: prize,
        };

        let info = mock_info(minter.as_ref(), &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(get_balance(deps.as_ref(), genesis), amount);
        assert_eq!(get_balance(deps.as_ref(), winner.clone()), prize);

        // Allows minting 0
        let msg = ExecuteMsg::Mint {
            recipient: winner.clone(),
            amount: Uint128::zero(),
        };
        let info = mock_info(minter.as_ref(), &[]);
        let env = mock_env();
        execute(deps.as_mut(), env, info, msg).unwrap();

        // but if it exceeds cap (even over multiple rounds), it fails
        // cap is enforced
        let msg = ExecuteMsg::Mint {
            recipient: winner,
            amount: Uint128::new(333_222_222),
        };
        let info = mock_info(minter.as_ref(), &[]);
        let env = mock_env();
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::CW20Base(CW20BaseError::CannotExceedCap {})
        );
    }

    #[test]
    fn others_cannot_mint() {
        let mut deps = mock_dependencies();

        let genesis = deps.api.addr_make("genesis").to_string();
        let minter = deps.api.addr_make("minter").to_string();
        let winner = deps.api.addr_make("winner").to_string();

        do_instantiate_with_minter(deps.as_mut(), &genesis, Uint128::new(1234), &minter, None);

        let msg = ExecuteMsg::Mint {
            recipient: winner,
            amount: Uint128::new(222),
        };
        let info = mock_info("anyone else", &[]);
        let env = mock_env();
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::CW20Base(CW20BaseError::Unauthorized {}));
    }

    #[test]
    fn minter_can_update_minter_but_not_cap() {
        let mut deps = mock_dependencies();

        let genesis = deps.api.addr_make("genesis").to_string();
        let minter = deps.api.addr_make("minter").to_string();

        let cap = Some(Uint128::from(3000000u128));
        do_instantiate_with_minter(deps.as_mut(), &genesis, Uint128::new(1234), &minter, cap);

        let new_minter = deps.api.addr_make("new_minter").to_string();
        let msg = ExecuteMsg::UpdateMinter {
            new_minter: Some(new_minter.clone()),
        };

        let info = mock_info(&minter, &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env.clone(), info, msg);
        assert!(res.is_ok());
        let query_minter_msg = QueryMsg::Minter {};
        let res = query(deps.as_ref(), env, query_minter_msg);
        let mint: MinterResponse = from_json(res.unwrap()).unwrap();

        // Minter cannot update cap.
        assert!(mint.cap == cap);
        assert!(mint.minter == new_minter)
    }

    #[test]
    fn others_cannot_update_minter() {
        let mut deps = mock_dependencies();

        let genesis = deps.api.addr_make("genesis").to_string();
        let minter = deps.api.addr_make("minter").to_string();
        let new_minter = deps.api.addr_make("new_minter").to_string();

        do_instantiate_with_minter(deps.as_mut(), &genesis, Uint128::new(1234), &minter, None);

        let msg = ExecuteMsg::UpdateMinter {
            new_minter: Some(new_minter),
        };

        let info = mock_info("not the minter", &[]);
        let env = mock_env();
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::CW20Base(CW20BaseError::Unauthorized {}));
    }

    #[test]
    fn unset_minter() {
        let mut deps = mock_dependencies();

        let genesis = deps.api.addr_make("genesis").to_string();
        let minter = deps.api.addr_make("minter").to_string();
        let winner = deps.api.addr_make("winner").to_string();

        let cap = None;
        do_instantiate_with_minter(deps.as_mut(), &genesis, Uint128::new(1234), &minter, cap);

        let msg = ExecuteMsg::UpdateMinter { new_minter: None };

        let info = mock_info(&minter, &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env.clone(), info, msg);
        assert!(res.is_ok());
        let query_minter_msg = QueryMsg::Minter {};
        let res = query(deps.as_ref(), env, query_minter_msg);
        let mint: Option<MinterResponse> = from_json(res.unwrap()).unwrap();

        // Check that mint information was removed.
        assert_eq!(mint, None);

        // Check that old minter can no longer mint.
        let msg = ExecuteMsg::Mint {
            recipient: winner,
            amount: Uint128::new(222),
        };
        let info = mock_info(&minter, &[]);
        let env = mock_env();
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::CW20Base(CW20BaseError::Unauthorized {}));
    }
}
