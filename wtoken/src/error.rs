use cosmwasm_std::StdError;
use cw20::Cw20ExecuteMsg;
pub use cw20_base::ContractError as CW20BaseError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("CW20Base error: {0}")]
    CW20Base(#[from] CW20BaseError),
    #[error("Not supported execution")]
    NotSupportedExecution(Cw20ExecuteMsg),
}

pub type ContractResult<T> = Result<T, ContractError>;
