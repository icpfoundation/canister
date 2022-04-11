use ic_cdk_macros::*;
use std::collections::HashMap;
mod authority;
mod group;
mod member;
mod operation;
mod project;
use authority::Authority;
use group::Group;
use member::Member;
use project::Project;
use std::sync::RwLock;
use ic_cdk::export::candid::{Deserialize, Nat};
#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref ProjectRef: u64 = 0;
    pub static ref GroupRef: u64 = 0;
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
   let group =  Group::new(
        1u64,
        "group1",
        "group1",
        Authority::ReadOnly,
        &project,
        &members,
    );
    group.storage();
}
