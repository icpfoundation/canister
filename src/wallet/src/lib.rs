use ic_cdk::export::candid::Principal;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;
#[derive(CandidType, Debug, Deserialize, Clone, PartialEq, Eq)]
struct Wallet {
    identity: Principal,
    describe: String,
}
type Wallet_Storage = HashMap<Principal, Vec<Wallet>>;

thread_local! {
    static WALLET_STORAGE: RefCell<Wallet_Storage> = RefCell::default();
}

#[update]
fn add_wallet(wallet: Wallet) {
    let caller = ic_cdk::caller();
    WALLET_STORAGE.with(|wallet_storage| {
        if let None = wallet_storage.borrow_mut().get_mut(&caller) {
            wallet_storage.borrow_mut().insert(caller, Vec::new());
        }
        if wallet_storage
            .borrow()
            .get(&caller)
            .unwrap()
            .contains(&wallet)
        {
            return;
        }
        wallet_storage
            .borrow_mut()
            .get_mut(&ic_cdk::caller())
            .unwrap()
            .push(wallet);
    });
}

#[query]
fn get_wallet() -> Option<Vec<Wallet>> {
    WALLET_STORAGE.with(|wallet_storage| wallet_storage.borrow().get(&ic_cdk::caller()).cloned())
}
