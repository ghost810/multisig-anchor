use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

use cosmwasm_std::{Addr, BlockInfo, CosmosMsg, Empty, StdError, StdResult, Storage,
    from_binary, to_binary, Binary, CanonicalAddr, QueryRequest,Uint128, WasmQuery, DepsMut,
};

use cw0::{Duration, Expiration};
use cw3::{Status, Vote};
use cw_storage_plus::{Item, Map, U64Key};
use cosmwasm_storage::to_length_prefixed;



pub fn load_token_balance(
    deps: DepsMut,
    contract_addr: String,
    account_addr: &CanonicalAddr,
) -> StdResult<Uint128> {
    // load balance form the token contract
    let res: Binary = deps
        .querier
        .query(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: contract_addr,
            key: Binary::from(concat(
                &to_length_prefixed(b"balance").to_vec(),
                account_addr.to_vec().as_slice(),
            )),
        }))
        .unwrap_or_else(|_| to_binary(&Uint128::zero()).unwrap());

    from_binary(&res)
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub anchor_address: String,
    pub market_address: String,
    pub recipient: String,
    pub required_weight: u64,
    pub total_weight: u64,
    pub max_voting_period: Duration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Proposal {
    pub title: String,
    pub description: String,
    pub expires: Expiration,
    pub msgs: Vec<CosmosMsg<Empty>>,
    pub status: Status,
    /// how many votes have already said yes
    pub yes_weight: u64,
    /// how many votes needed to pass
    pub required_weight: u64,
}

impl Proposal {
    pub fn current_status(&self, block: &BlockInfo) -> Status {
        let mut status = self.status;

        // if open, check if voting is passed or timed out
        if status == Status::Open && self.yes_weight >= self.required_weight {
            status = Status::Passed;
        }
        if status == Status::Open && self.expires.is_expired(block) {
            status = Status::Rejected;
        }

        status
    }
}

// we cast a ballot with our chosen vote and a given weight
// stored under the key that voted
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Ballot {
    pub weight: u64,
    pub vote: Vote,
}

// unique items
pub const CONFIG: Item<Config> = Item::new("config");
pub const PROPOSAL_COUNT: Item<u64> = Item::new("proposal_count");
pub const PRINCIPLE_AMOUNT: Item<Uint128> = Item::new("principle_amount");

// multiple-item maps
pub const VOTERS: Map<&Addr, u64> = Map::new("voters");
pub const PROPOSALS: Map<U64Key, Proposal> = Map::new("proposals");
pub const BALLOTS: Map<(U64Key, &Addr), Ballot> = Map::new("ballots");

pub fn next_id(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = PROPOSAL_COUNT.may_load(store)?.unwrap_or_default() + 1;
    PROPOSAL_COUNT.save(store, &id)?;
    Ok(id)
}

pub fn parse_id(data: &[u8]) -> StdResult<u64> {
    match data[0..8].try_into() {
        Ok(bytes) => Ok(u64::from_be_bytes(bytes)),
        Err(_) => Err(StdError::generic_err(
            "Corrupted data found. 8 byte expected.",
        )),
    }
}
