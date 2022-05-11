use ic_cdk::export::candid::Principal;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;
#[derive(CandidType, Debug, Deserialize, Clone)]
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
    WALLET_STORAGE.with(|wallet_storage| {
        if let None = wallet_storage.borrow_mut().get_mut(&ic_cdk::caller()) {
            wallet_storage
                .borrow_mut()
                .insert(ic_cdk::caller(), Vec::new());
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
