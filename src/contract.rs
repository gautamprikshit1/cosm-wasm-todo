use std::ops::Add;
use std::usize;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Order};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ListResponse, EntryResponse};
use crate::state::{Config, Entry, Priority, Status, CONFIG, ENTRY_SEQ, LIST};

const CONTRACT_NAME: &str = "crates.io:todo-list";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .and_then(|addr_string| deps.api.addr_validate(addr_string.as_str()).ok())
        .unwrap_or(info.sender);

    let config = Config {
        owner: owner.clone(),
    };

    CONFIG.save(deps.storage, &config)?;
    ENTRY_SEQ.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::NewEntry {
            description,
            priority,
        } => execute_create_new_entry(deps, info, description, priority),
        ExecuteMsg::UpdateEntry {
            id,
            description,
            status,
            priority,
        } => execute_update_entry(deps, info, id, description, status, priority),
        ExecuteMsg::DeleteEntry { id } => execute_delete_entry(deps, info, id),
    }
}

pub fn execute_delete_entry(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    ) -> Result<Response, ContractError> {
    let owner = CONFIG.load(deps.storage)?.owner;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    LIST.remove(deps.storage, id);
    Ok(Response::new()
       .add_attribute("method", "execute_delete_entry")
       .add_attribute("deleted_entry_id", id.to_string())
       )
}

pub fn execute_update_entry(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    description: Option<String>,
    status: Option<Status>,
    priority: Option<Priority>,
) -> Result<Response, ContractError> {
    let owner = CONFIG.load(deps.storage)?.owner;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    let entry = LIST.load(deps.storage, id)?;
    let updated_entry = Entry {
        id,
        description: description.unwrap_or(entry.description),
        status: status.unwrap_or(entry.status),
        priority: priority.unwrap_or(entry.priority),
    };

    LIST.save(deps.storage, id, &updated_entry)?;
    Ok(Response::new()
       .add_attribute("method", "execute_update_entry")
       .add_attribute("updated_entry_id", id.to_string())
       )
}

pub fn execute_create_new_entry(
    deps: DepsMut,
    info: MessageInfo,
    description: String,
    priority: Option<Priority>,
) -> Result<Response, ContractError> {
    let owner = CONFIG.load(deps.storage)?.owner;

    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    let id = ENTRY_SEQ.update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id.add(1)))?;

    let new_entry = Entry {
        id,
        description,
        priority: priority.unwrap_or(Priority::None),
        status: Status::ToDo,
    };

    LIST.save(deps.storage, id, &new_entry)?;
    Ok(Response::new()
        .add_attribute("method", "execute_new_entry")
        .add_attribute("new_entry_id", id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryList { start_after, limit } => to_binary(&query_list(deps, start_after, limit)?),
        QueryMsg::QueryEntry { id } => to_binary(&query_entry(deps, id)?),
    }
}

pub fn query_entry(deps: Deps, id: u64) -> StdResult<EntryResponse> {
    let entry = LIST.load(deps.storage, id)?;
    Ok(EntryResponse {
        id: entry.id,
        description: entry.description,
        status: entry.status,
        priority: entry.priority,
    })
}

pub fn query_list(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> StdResult<ListResponse> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let enteries: StdResult<Vec<_>> = LIST
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();
    let result = ListResponse {
        enteries: enteries?.into_iter().map(|l| l.1.into()).collect(),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {}
