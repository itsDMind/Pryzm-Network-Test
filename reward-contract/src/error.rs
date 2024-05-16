use cosmwasm_std::{Addr, OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("{0}")]
    OverflowError(#[from] OverflowError),
    #[error("{address} is not a sender of wtoken nor usdsim")]
    InvalidSender { address: Addr },
    #[error("zero amount received")]
    ZeroAmountReceived,
}

pub type ContractResult<T> = Result<T, ContractError>;
