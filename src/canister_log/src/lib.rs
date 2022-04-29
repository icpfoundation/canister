use ic_cdk::export::Principal;
use ic_cdk_macros::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;
mod log;
use log::Log;
use std::cell::RefCell;
type Log_Storage = HashMap<Principal, BTreeMap<u64, Log>>;
thread_local! {
    static LOG_STORAGE: RefCell<Log_Storage> = RefCell::default();
}

#[update]
fn create_log(operator: Principal, log: Vec<u8>) {
    let new_log = log::Log::new(operator, log);
    LOG_STORAGE.with(|log_storage| {
        if let Some(data) = log_storage.borrow_mut().get_mut(&operator) {
            data.insert(new_log.create_time, new_log);
            return;
        }
        log_storage.borrow_mut().insert(operator, BTreeMap::new());
        log_storage
            .borrow_mut()
            .get_mut(&operator)
            .unwrap()
            .insert(new_log.create_time, new_log);
    });
}

#[query]
fn get_log(operator: Principal) -> Option<Vec<Vec<String>>> {
    LOG_STORAGE.with(|log_storage| {
        if let None = log_storage.borrow().get(&operator) {
            return None;
        }

        let result: Vec<Vec<String>> = log_storage
            .borrow()
            .get(&operator)
            .unwrap()
            .values()
            .map(|x| {
                let res = rlp::decode_list::<String>(&x.info);
                res
            })
            .collect();
        Some(result)
    })
}
