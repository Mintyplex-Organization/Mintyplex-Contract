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

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::MintyPlexContract;
use crate::{ContractError, Extension};

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

#[test]
fn minting() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let token_id = "maggi".to_string();
    let token_uri = "http://maggi.com.ng/maggi.png".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: String::from("cook"),
        token_uri: Some(token_uri.clone()),
        extension: None,
    };

    let allowed = mock_info(MINTER, &[]);
    let _ = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg)
        .unwrap();

    // ensure num tokens increases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(1, count.count);

    // unknown nft returns error
    let _ = contract
        .nft_info(deps.as_ref(), "unknown".to_string())
        .unwrap_err();

    // this nft info is correct
    let info = contract.nft_info(deps.as_ref(), token_id.clone()).unwrap();
    assert_eq!(
        info,
        NftInfoResponse::<Extension> {
            token_uri: Some(token_uri),
            extension: None,
        }
    );

    // owner info is correct
    let owner = contract
        .owner_of(deps.as_ref(), mock_env(), token_id.clone(), true)
        .unwrap();
    assert_eq!(
        owner,
        OwnerOfResponse {
            owner: String::from("cook"),
            approvals: vec![],
        }
    );

    // Cannot mint same token_id again
    let mint_msg2 = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: String::from("hercules"),
        token_uri: None,
        extension: None,
    };

    let allowed = mock_info(MINTER, &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg2)
        .unwrap_err();
    assert_eq!(err, ContractError::Claimed {});

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id], tokens.tokens);
}

#[test]
fn transferring_nft() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let token_id = "fish".to_string();
    let token_uri = "http://google.com/png".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: String::from("odogwu"),
        token_uri: Some(token_uri),
        extension: None,
    };

    let minter = mock_info(MINTER, &[]);

    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // Testing that a random address cannot transfer.
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: token_id.clone(),
    };

    let err = contract
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // Owner can send NFT
    let owner_addr = mock_info("odogwu", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: token_id.clone(),
    };

    let res = contract
        .execute(deps.as_mut(), mock_env(), owner_addr, transfer_msg)
        .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", "odogwu")
            .add_attribute("recipient", "random")
            .add_attribute("token_id", token_id)
    );
}

#[test]
fn approving_revoking() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // mint a token
    let token_id = "minty".to_string();
    let token_uri = "https://mintyplex.com/minty".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: String::from("people"),
        token_uri: Some(token_uri),
        extension: None,
    };

    let minter = mock_info(MINTER, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // Token owner shows in approval query
    let res = contract
        .approval(
            deps.as_ref(),
            mock_env(),
            token_id.clone(),
            String::from("people"),
            false,
        )
        .unwrap();

    assert_eq!(
        res,
        ApprovalResponse {
            approval: Approval {
                spender: String::from("people"),
                expires: Expiration::Never {}
            }
        }
    );

    //  give odogwu transferring power

    let approve_msg = ExecuteMsg::Approve {
        spender: String::from("odogwu"),
        token_id: token_id.clone(),
        expires: None,
    };

    let owner = mock_info("people", &[]);

    let res = contract
        .execute(deps.as_mut(), mock_env(), owner, approve_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", "people")
            .add_attribute("spender", "odogwu")
            .add_attribute("token_id", token_id.clone())
    );

    // test approval query
    let res = contract
        .approval(
            deps.as_ref(),
            mock_env(),
            token_id.clone(),
            String::from("people"),
            true,
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovalResponse {
            approval: Approval {
                spender: String::from("people"),
                expires: Expiration::Never {}
            }
        }
    );

    // random can now transfer
    let random = mock_info("people", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: token_id.clone(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap();

    let query_msg = QueryMsg::OwnerOf {
        token_id: token_id.clone(),
        include_expired: None,
    };

    let res: OwnerOfResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), query_msg.clone())
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![]
        }
    );

    // Approve, revoke, and check for empty, to test revoke
    let approve_msg = ExecuteMsg::Approve {
        spender: String::from("mua"),
        token_id: token_id.clone(),
        expires: None,
    };

    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner.clone(), approve_msg)
        .unwrap();

    let revoke_msg = ExecuteMsg::Revoke {
        spender: String::from("mua"),
        token_id: token_id.clone(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), owner, revoke_msg)
        .unwrap();

    // Approvals are now removed and cleared
    let res: OwnerOfResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), query_msg)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![]
        }
    );
}
