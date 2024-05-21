use cosmwasm_std::{Addr, OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("{0}")]
    OverflowError(#[from] OverflowError),
    #[error("{address} is not wtoken address")]
    InvalidWtokenAddress { address: Addr },
    #[error("zero amount received")]
    ZeroAmountReceived,
    #[error("No usdsim tokens sent")]
    NoUsdsimTokensSent,
}

pub type ContractResult<T> = Result<T, ContractError>;
