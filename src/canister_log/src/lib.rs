use ic_cdk::export::Principal;
use ic_cdk_macros::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;
mod log;
use ic_cdk::export::candid::{CandidType, Deserialize};
use log::Log;
use std::cell::RefCell;
use std::hash::Hash;
#[derive(Hash, PartialEq, Eq, Clone, CandidType, Debug, Deserialize)]
struct User {
    identity: Principal,
    group_id: u64,
}
type Log_Storage = HashMap<User, BTreeMap<u64, Vec<Log>>>;
const PAGE_SIZE: usize = 20;
static mut OWNER: Principal = Principal::from_slice(&[0]);
static mut MANAGE_CANISTER: Principal = Principal::from_slice(&[0]);
thread_local! {
    static LOG_STORAGE: RefCell<Log_Storage> = RefCell::default();

}

#[init]
fn init(manage_canister: Principal) {
    unsafe {
        OWNER = ic_cdk::caller();
        MANAGE_CANISTER = manage_canister;
    }
}

#[update]
pub fn update_manage_canister(mange_canister: Principal) {
    let caller = ic_cdk::api::caller();
    unsafe {
        if OWNER != caller {
            ic_cdk::trap("invalid identity");
        }
        MANAGE_CANISTER = mange_canister;
    }
}

#[update]
fn create_log(
    user: Principal,
    group_id: u64,
    operator: Principal,
    action: log::Action,
    log: Vec<u8>,
) {
    let caller = ic_cdk::api::caller();
    unsafe {
        if caller != MANAGE_CANISTER {
            return;
        }
    }

    let user = User {
        identity: user,
        group_id: group_id,
    };
    let new_log = log::Log::new(operator, action, log);
    LOG_STORAGE.with(|log_storage| {
        let mut storage = log_storage.borrow_mut();
        match storage.get_mut(&user) {
            None => {
                storage.insert(user.clone(), BTreeMap::new());
                storage.get_mut(&user).unwrap().insert(1, vec![new_log]);
            }
            Some(data) => {
                let page_size = data.len() as u64;
                let log_data = data.get_mut(&page_size).unwrap();
                if log_data.len() < PAGE_SIZE {
                    log_data.push(new_log)
                } else {
                    let new_page = page_size + 1;
                    data.insert(new_page, vec![new_log]);
                }
            }
        }
    });
}

#[query]
fn get_log(
    account: Principal,
    group_id: u64,
    page: u64,
) -> Option<Vec<(Principal, u64, log::Action, Vec<String>)>> {
    let user = User {
        identity: account,
        group_id: group_id,
    };
    LOG_STORAGE.with(|log_storage| {
        if let None = log_storage.borrow().get(&user) {
            return None;
        }
        let result: Vec<(Principal, u64, log::Action, Vec<String>)> = log_storage
            .borrow()
            .get(&user)
            .unwrap()
            .get(&page)
            .unwrap()
            .iter()
            .map(|x| {
                let info = rlp::decode_list::<String>(&x.info);
                (x.operator, x.create_time, x.action.clone(), info)
            })
            .collect();
        Some(result)
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    LOG_STORAGE.with(|log_storage| {
        let data_storage: Vec<(User, Vec<(u64, Vec<Log>)>)> = log_storage
            .borrow()
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter().map(|(idx, data)| (*idx, data.clone())).collect(),
                )
            })
            .collect();
        ic_cdk::storage::stable_save((data_storage,)).expect("stable_save failed");
    })
}

#[post_upgrade]
fn post_update() {
    let data_storage: (Vec<(User, Vec<(u64, Vec<Log>)>)>,) =
        ic_cdk::storage::stable_restore().expect("data recovery failed");
    let data_storage: Vec<(User, BTreeMap<u64, Vec<Log>>)> = data_storage
        .0
        .iter()
        .map(|(k, v)| (k.clone(), v.clone().into_iter().collect()))
        .collect();
    let data_storage: Log_Storage = data_storage.into_iter().collect();
    LOG_STORAGE.with(|log_storage| {
        *log_storage.borrow_mut() = data_storage;
    });
}
