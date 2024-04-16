use cosmwasm_std::{Decimal, DecimalRangeExceeded, OverflowError, StdError};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid Collateral Asset")]
    InvalidCollateralAsset { denom: String },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Resultant health factor lower than min allowed")]
    BreaksHealthFactor {
        health_factor_value: Decimal,
        min_value: Decimal,
    },

    #[error("Token addresses and price feed ids lengths don't match")]
    TokenAddressesAndPriceFeedIdsLengthsDontMatch {},

    #[error("Health factor from liquidated user not improved")]
    HealthFactorNotImproved {},

    #[error("Health factor is ok, user cannot be liquidated")]
    HealthFactorOk {},

    #[error("Missing Native Asset Funds")]
    MissingNativeFunds { denom: String },

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Invalid expiration")]
    InvalidExpiration {},

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Allowance is expired")]
    Expired {},

    #[error("No allowance for this account")]
    NoAllowance {},

    #[error("Minting cannot exceed the cap")]
    CannotExceedCap {},

    #[error("Duplicate initial balance addresses")]
    DuplicateInitialBalanceAddresses {},
}

impl From<cw20_base::ContractError> for ContractError {
    fn from(err: cw20_base::ContractError) -> Self {
        match err {
            cw20_base::ContractError::Std(error) => ContractError::Std(error),
            cw20_base::ContractError::Unauthorized {} => ContractError::Unauthorized {},
            cw20_base::ContractError::CannotSetOwnAccount {} => {
                ContractError::CannotSetOwnAccount {}
            }
            cw20_base::ContractError::InvalidExpiration {} => ContractError::InvalidExpiration {},
            cw20_base::ContractError::InvalidZeroAmount {} => ContractError::InvalidZeroAmount {},
            cw20_base::ContractError::Expired {} => ContractError::Expired {},
            cw20_base::ContractError::NoAllowance {} => ContractError::NoAllowance {},
            cw20_base::ContractError::CannotExceedCap {} => ContractError::CannotExceedCap {},
            // This should never happen, as this contract doesn't use logo
            cw20_base::ContractError::LogoTooBig {}
            | cw20_base::ContractError::InvalidPngHeader {}
            | cw20_base::ContractError::InvalidXmlPreamble {} => {
                ContractError::Std(StdError::generic_err(err.to_string()))
            }
            cw20_base::ContractError::DuplicateInitialBalanceAddresses {} => {
                ContractError::DuplicateInitialBalanceAddresses {}
            }
        }
    }
}

impl From<DecimalRangeExceeded> for ContractError {
    fn from(error: DecimalRangeExceeded) -> Self {
        ContractError::Std(StdError::generic_err(error.to_string()))
    }
}

impl From<OverflowError> for ContractError {
    fn from(error: OverflowError) -> Self {
        ContractError::Std(StdError::generic_err(error.to_string()))
    }
}

impl From<FromHexError> for ContractError {
    fn from(error: FromHexError) -> Self {
        ContractError::Std(StdError::generic_err(error.to_string()))
    }
}
