use cosmwasm_schema::write_api;

use cosmwasm_std::Empty;
use mintyplex_contract::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg<Empty>,
        query: QueryMsg<Empty>,
    }
}
