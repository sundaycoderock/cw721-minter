use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use cw721_metadata_onchain::Metadata;

#[cw_serde]
pub enum ExecuteMsg {
    SetNftContract {
        address: String,
    },
    MintNft {
        token_id: String,
        owner: String,
        token_uri: Option<String>,
        extension: Option<Metadata>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StateResponse)]
    GetState {},
}

#[cw_serde]
pub struct StateResponse {
    pub admin: Addr,
    pub nft_contract: Addr,
}
