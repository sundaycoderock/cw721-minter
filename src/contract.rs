#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
// use cw2::set_contract_version;

use crate::msg::StateResponse;
use crate::msg::{ExecuteMsg, QueryMsg};
use crate::state::State;
use crate::{state::STATE, ContractError};
use cw721_base::ExecuteMsg as Cw721BaseExecuteMsg;
use cw721_metadata_onchain::Metadata;

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw721-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    let state = State {
        admin: deps.api.addr_validate(&info.sender.as_str()).unwrap(),
        nft_contract: Addr::unchecked(""),
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetNftContract { address } => exec::set_nft_contract(deps, info, address),
        ExecuteMsg::MintNft {
            token_id,
            owner,
            token_uri,
            extension,
        } => exec::mint_nft(deps, info, token_id, owner, token_uri, extension),
    }
}

mod exec {
    use super::*;

    pub fn set_nft_contract(
        deps: DepsMut,
        info: MessageInfo,
        address: String,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;

        if info.sender != state.admin {
            return Err(ContractError::Unauthorized {});
        }

        let state = State {
            admin: state.admin.clone(),
            nft_contract: deps.api.addr_validate(&address).unwrap(),
        };

        STATE.save(deps.storage, &state)?;

        Ok(Response::default())
    }

    pub fn mint_nft(
        deps: DepsMut,
        info: MessageInfo,
        token_id: String,
        owner: String,
        token_uri: Option<String>,
        extension: Option<Metadata>,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;

        if state.nft_contract == Addr::unchecked("") {
            return Err(ContractError::EmptyNftAddress {});
        }

        for coin in &info.funds {
            if coin.denom != "ujunox" {
                return Err(ContractError::UnmatchDenom {});
            } else if coin.denom == "ujunox" && coin.amount == Uint128::new(0) {
                return Err(ContractError::InsufficientFunds {});
            }
        }

        let mint_nft_msg: Cw721BaseExecuteMsg<Option<Metadata>, Empty> =
            Cw721BaseExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
                extension,
            };

        let _exec_mint_nft_msg = WasmMsg::Execute {
            contract_addr: state.nft_contract.into_string(),
            msg: to_binary(&mint_nft_msg).unwrap(),
            funds: vec![],
        };

        let transfer_funds_msg = BankMsg::Send {
            to_address: state.admin.into(),
            amount: info.funds,
        };

        let _exec_transfer_funds_msg: CosmosMsg = transfer_funds_msg.into();

        Ok(Response::default())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => {
            let state_response = query::query_state(deps).unwrap();
            let result = to_binary(&state_response)?;
            Ok(result)
        }
    }
}

mod query {

    use super::*;

    pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
        let state = STATE.load(deps.storage)?;

        Ok(StateResponse {
            admin: state.admin,
            nft_contract: state.nft_contract,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{
        coins, from_binary,
        testing::{mock_dependencies, mock_env, mock_info},
    };

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let info = mock_info("sender", &coins(1000, "test"));

        let msg: Empty = Default::default();

        let result = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, result.messages.len());

        let info = mock_info("sender", &coins(1000, "test"));

        let result = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let value: StateResponse = from_binary(&result).unwrap();
        assert_eq!(info.sender, value.admin);
    }

    #[test]
    fn set_nft_contract() {
        // instantiate
        let mut deps = mock_dependencies();
        let info = mock_info("sender", &coins(1000, "test"));
        let msg: Empty = Default::default();
        let result = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, result.messages.len());

        // non admin cannot set nft contract
        let info = mock_info("impostor", &coins(1000, "test"));
        let msg = ExecuteMsg::SetNftContract {
            address: "scam".to_string(),
        };
        let result = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(result.is_err());

        // admin can set nft contract
        let info = mock_info("sender", &coins(1000, "test"));
        let msg = ExecuteMsg::SetNftContract {
            address: "nft_contract".to_string(),
        };
        let result = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(result.is_ok());
    }

    #[test]
    fn mint_nft() {
        // instantiate
        let mut deps = mock_dependencies();
        let info = mock_info("sender", &coins(1000, "test"));
        let msg: Empty = Default::default();
        let result = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, result.messages.len());

        // if nft contract not set
        let msg = QueryMsg::GetState {};
        let result = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: StateResponse = from_binary(&result).unwrap();
        assert_eq!(Addr::unchecked(""), value.nft_contract);

        // funds denom not match
        let info = mock_info("sender", &coins(1000, "fake"));
        let msg = ExecuteMsg::MintNft {
            token_id: "1".to_string(),
            owner: "newowner".to_string(),
            token_uri: Some("ipfs".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                ..Metadata::default()
            }),
        };
        let result = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(result.is_err());

        // insufficient funds
        let info = mock_info("sender", &coins(0, "test"));
        let msg = ExecuteMsg::MintNft {
            token_id: "1".to_string(),
            owner: "newowner".to_string(),
            token_uri: Some("ipfs".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                ..Metadata::default()
            }),
        };
        let result = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(result.is_err());

        // user mint nft
        let info = mock_info("sender", &coins(1000, "ujunox"));
        let msg = ExecuteMsg::SetNftContract {
            address: "nft_contract".to_string(),
        };
        let result = execute(deps.as_mut(), mock_env(), info, msg);
        assert!(result.is_ok());

        let info = mock_info("sender", &coins(1000, "ujunox"));
        let token_id = "Enterprise";
        let token_uri = Some("https://starships.example.com/Starship/Enterprise.json".into());
        let extension = Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        });
        let exec_msg = ExecuteMsg::MintNft {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: token_uri.clone(),
            extension: extension.clone(),
        };
        let result = execute(deps.as_mut(), mock_env(), info, exec_msg);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
