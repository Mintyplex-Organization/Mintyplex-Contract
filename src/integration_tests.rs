#![cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

use cosmwasm_std::{from_json, DepsMut, Empty, Response};

use cw721::{
    Approval, ApprovalResponse, ContractInfoResponse, Cw721Query, Expiration, NftInfoResponse,
    OwnerOfResponse,
};
use cw_ownable::OwnershipError;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::MintyPlexContract;
use crate::{ContractError, Extension};

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

#[test]
fn query_tokens_by_owner() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());
    let minter = mock_info(MINTER, &[]);

    let token_id1 = "garri".to_string();
    let chidinma = String::from("chidinma");
    let token_id2 = "akpu".to_string();
    let amaka = String::from("amaka");
    let token_id3 = "beans".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id1.clone(),
        owner: chidinma.clone(),
        token_uri: None,
        extension: None,
    };

    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id2.clone(),
        owner: amaka.clone(),
        token_uri: None,
        extension: None,
    };

    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id3.clone(),
        owner: chidinma.clone(),
        token_uri: None,
        extension: None,
    };

    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    // get all tokens in order:
    let expected = vec![token_id2.clone(), token_id3.clone(), token_id1.clone()];
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(&expected, &tokens.tokens);

    // get by owner
    let by_amaka = vec![token_id2];
    let by_chidnma = vec![token_id1, token_id3];

    // All tokens by owner
    let tokens = contract
        .tokens(deps.as_ref(), chidinma.clone(), None, None)
        .unwrap();
    let reversed_tokens: Vec<_> = tokens.tokens.iter().rev().cloned().collect();
    assert_eq!(&by_chidnma, &reversed_tokens);

    let tokens = contract
        .tokens(deps.as_ref(), amaka.clone(), None, None)
        .unwrap();
    assert_eq!(&by_amaka, &tokens.tokens);
}
