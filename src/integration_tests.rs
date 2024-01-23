// #[cfg(test)]
// mod tests {
//     use crate::helpers::CwTemplateContract;
//     use crate::msg::InstantiateMsg;
//     use cosmwasm_std::{Addr, Coin, Empty, Uint128};
//     use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

//     pub fn contract_template() -> Box<dyn Contract<Empty>> {
//         let contract = ContractWrapper::new(
//             crate::execute::execute,
//             crate::execute::instantiate,
//             crate::execute::query,
//         );
//         Box::new(contract)
//     }

//     const USER: &str = "USER";
//     const ADMIN: &str = "ADMIN";
//     const NATIVE_DENOM: &str = "denom";

//     fn mock_app() -> App {
//         AppBuilder::new().build(|router, _, storage| {
//             router
//                 .bank
//                 .init_balance(
//                     storage,
//                     &Addr::unchecked(USER),
//                     vec![Coin {
//                         denom: NATIVE_DENOM.to_string(),
//                         amount: Uint128::new(1),
//                     }],
//                 )
//                 .unwrap();
//         })
//     }

//     fn proper_instantiate() -> (App, CwTemplateContract) {
//         let mut app = mock_app();
//         let cw_template_id = app.store_code(contract_template());

//         let msg = InstantiateMsg { count: 1i32 };
//         let cw_template_contract_addr = app
//             .instantiate_contract(
//                 cw_template_id,
//                 Addr::unchecked(ADMIN),
//                 &msg,
//                 &[],
//                 "test",
//                 None,
//             )
//             .unwrap();

//         let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

//         (app, cw_template_contract)
//     }

//     mod count {
//         use super::*;
//         use crate::msg::ExecuteMsg;

//         #[test]
//         fn count() {
//             let (mut app, cw_template_contract) = proper_instantiate();

//             let msg = ExecuteMsg::Increment {};
//             let cosmos_msg = cw_template_contract.call(msg).unwrap();
//             app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
//         }
//     }
// }

#![cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

use cosmwasm_std::{
    from_json, to_json_binary, Addr, Coin, CosmosMsg, DepsMut, Empty, Response, StdError, WasmMsg,
};

use cw721::{
    Approval, ApprovalResponse, ContractInfoResponse, Cw721Query, Cw721ReceiveMsg, Expiration,
    NftInfoResponse, OperatorResponse, OperatorsResponse, OwnerOfResponse,
};
use cw_ownable::OwnershipError;

use crate::msg::InstantiateMsg;
use crate::state::MintyPlexContract;
use crate::Extension;

// use crate::{
//     ContractError, Cw721Contract, ExecuteMsg, Extension, InstantiateMsg, MinterResponse, QueryMsg,
// };

const MINTER: &str = "merlin";
const CONTRACT_NAME: &str = "Magic Power";
const SYMBOL: &str = "MGK";

fn setup_contract(deps: DepsMut<'_>) -> MintyPlexContract<'static, Extension, Empty, Empty, Empty> {
    let contract = MintyPlexContract::default();
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        // minter: Some(String::from(MINTER)),
        // withdraw_address: None,
    };
    let info = mock_info("creator", &[]);
    let res = contract.instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    contract
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies();
    let contract = MintyPlexContract::<Extension, Empty, Empty, Empty>::default();

    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        // minter: Some(String::from(MINTER)),
        // withdraw_address: Some(String::from(MINTER)),
    };
    let info = mock_info("creator", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = contract
        .instantiate(deps.as_mut(), mock_env(), info, msg)
        .unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    // let res = contract.minter(deps.as_ref()).unwrap();
    // assert_eq!(Some(MINTER.to_string()), res.minter);
    let info = contract.contract_info(deps.as_ref()).unwrap();
    assert_eq!(
        info,
        ContractInfoResponse {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
        }
    );

    // let withdraw_address = contract
    //     .withdraw_address
    //     .may_load(deps.as_ref().storage)
    //     .unwrap();
    // assert_eq!(Some(MINTER.to_string()), withdraw_address);

    // let count = contract.num_tokens(deps.as_ref()).unwrap();
    // assert_eq!(0, count.count);

    // // list the token_ids
    // let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    // assert_eq!(0, tokens.tokens.len());
}
