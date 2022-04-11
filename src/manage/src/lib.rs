use ic_cdk_macros::*;
use std::collections::HashMap;
mod authority;
mod member;
mod group;
mod project;
use project::Project;
use group::Group;
use std::sync::RwLock;
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


