mod error;
pub mod execute;
pub mod helpers;
pub mod integration_tests;
pub mod msg;
pub mod query;
pub mod state;
use cosmwasm_std::Empty;

pub use crate::error::ContractError;

pub type Extension = Option<Empty>;

// Version info for migration
pub const CONTRACT_NAME: &str = "crates.io:mintyplex-contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod entry {
    use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response};

    use self::{
        msg::{ExecuteMsg, InstantiateMsg},
        state::MintyPlexContract,
    };

    use super::*;
    #[cfg(not(feature = "library"))]
    use cosmwasm_std::entry_point;

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let tract = MintyPlexContract::<Extension, Empty, Empty, Empty>::default();
        tract.instantiate(deps, env, info, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<Extension>,
    ) -> Result<Response, ContractError> {
        let tract = MintyPlexContract::<Extension, Empty, Empty, Empty>::default();
        tract.execute(deps, env, info, msg)
    }
}
