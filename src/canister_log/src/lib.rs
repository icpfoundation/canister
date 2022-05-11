use ic_cdk::export::Principal;
use ic_cdk_macros::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;
mod log;
use log::Log;
use std::cell::RefCell;
type Log_Storage = HashMap<Principal, BTreeMap<u64, Vec<Log>>>;
const PAGE_SIZE: usize = 30;
thread_local! {
    static LOG_STORAGE: RefCell<Log_Storage> = RefCell::default();
}

#[update]
fn create_log(operator: Principal, log: Vec<u8>) {
    let new_log = log::Log::new(operator, log);
    LOG_STORAGE.with(|log_storage| {
        let mut storage = log_storage.borrow_mut();
        match storage.get_mut(&operator) {
            None => {
                storage.insert(operator, BTreeMap::new());
                storage.get_mut(&operator).unwrap().insert(1, vec![new_log]);
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
fn get_log(operator: Principal, page: u64) -> Option<Vec<Vec<String>>> {
    LOG_STORAGE.with(|log_storage| {
        if let None = log_storage.borrow().get(&operator) {
            return None;
        }

        let result: Vec<Vec<String>> = log_storage
            .borrow()
            .get(&operator)
            .unwrap()
            .get(&page)
            .unwrap()
            .iter()
            .map(|x| {
                let res = rlp::decode_list::<String>(&x.info);
                res
            })
            .collect();
        Some(result)
    })
}
