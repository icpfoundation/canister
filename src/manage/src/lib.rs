use ic_cdk_macros::*;
use std::collections::HashMap;
mod authority;
mod group;
mod manage;
mod member;
mod operation;
mod project;
mod types;
use authority::Authority;
use candid::CandidType;
use group::Group;
use ic_cdk::export::candid::{Deserialize, Nat};
use ic_cdk::export::Principal;
use manage::{CanisterSettings, ManageCanister};
use member::Member;
use project::Project;
use std::sync::RwLock;
#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref ProjectNonce: u64 = 0;
    pub static ref GroupNonce: u64 = 0;
    pub static ref ProjectStorage: RwLock<HashMap<u64, Project>> = RwLock::new(HashMap::new());
    pub static ref GroupStorage: RwLock<HashMap<u64, Group>> = RwLock::new(HashMap::new());
}
#[init]
fn init() {}

#[query]
fn get_group(group_id: u64) -> Option<Group> {
    match GroupStorage.read().unwrap().get(&group_id) {
        None => return None,
        Some(group) => return Some(group.clone()),
    }
}

#[update]
fn mock_test_add_group() {
    let members: Vec<Member> = vec![];
    let project: Vec<Project> = vec![Project::new(
        1u64,
        "project1",
        "project1",
        ic_cdk::api::caller(),
        "https://github.com/icpfoundation/chain-cloud",
        Authority::ReadOnly,
        1u64,
        &members,
    )];
    let group = Group::new(
        1u64,
        "group1",
        "group1",
        Authority::ReadOnly,
        &project,
        &members,
    );
    group.storage().unwrap();
}

#[update]
async fn mock_test_set_controllers(canister_id: Principal) ->Result<(), String>{
    let controllers: Option<Vec<Principal>> = Some(vec![ic_cdk::api::caller()]);
    let compute_allocation: Nat = "0".parse().unwrap();
    let memory_allocation: Nat = "0".parse().unwrap();
    let freezing_threshold: Nat = "2_592_000".parse().unwrap();

    let canister_settings = CanisterSettings::new(
        controllers,
        Some(compute_allocation),
        Some(memory_allocation),
        Some(freezing_threshold),
    );
    let mange_canister = ManageCanister::new(canister_id, canister_settings);
    mange_canister.set_controller().await
}
