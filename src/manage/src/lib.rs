use ic_cdk_macros::*;
use std::collections::HashMap;
mod authority;
mod group;
mod manage;
mod member;
#[macro_use]
mod operation;
mod project;
mod types;
mod user;
use authority::Authority;
use candid::CandidType;
use group::Group;
use ic_cdk::export::candid::{Deserialize, Nat};
use ic_cdk::export::Principal;
use manage::{CanisterSettings, CanisterStatusResponse, InstallCodeMode, ManageCanister};
use member::Member;
use project::Project;
use std::sync::RwLock;
use types::Profile;
use user::User;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref ProjectNonce: u64 = 0;
    pub static ref GroupNonce: u64 = 0;
    pub static ref ProjectStorage: RwLock<HashMap<u64, Project>> = RwLock::new(HashMap::new());
    pub static ref GroupStorage: RwLock<HashMap<u64, Group>> = RwLock::new(HashMap::new());
    pub static ref UserStorage: RwLock<HashMap<Principal, User>> = RwLock::new(HashMap::new());
}
#[init]
fn init() {}

#[update]
async fn get_canister_status(
    user: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<CanisterStatusResponse, String> {
    User::get_canister_status(user, group_id, project_id, canister).await
}

#[update]
fn add_user(name: String, profile: Profile) -> Result<(), String> {
    let user = User::new(name, profile, ic_cdk::caller());
    user.storage()
}

#[update]
async fn add_group(group: Group) -> Result<(), String> {
    User::add_group(ic_cdk::caller(), group).await
}

#[update]
async fn remove_group(group_id: u64) -> Result<(), String> {
    User::remove_group(ic_cdk::caller(), group_id).await
}

#[query]
fn get_user_info(user: Principal) -> Result<User, String> {
    User::get_user_info(user)
}

#[update]
async fn add_project(group_id: u64, project: Project) -> Result<(), String> {
    User::add_project(ic_cdk::caller(), group_id, project).await
}

#[update]
async fn remove_project(group_id: u64, project_id: u64) -> Result<(), String> {
    User::remove_project(ic_cdk::caller(), group_id, project_id).await
}

#[update]
async fn add_group_member(group_id: u64, member: Member) -> Result<(), String> {
    User::add_group_member(ic_cdk::caller(), group_id, member).await
}

#[update]
async fn remove_group_member(group_id: u64, member: Principal) -> Result<(), String> {
    User::remove_group_member(ic_cdk::caller(), group_id, member).await
}

#[update]
async fn add_project_member(
    user: Principal,
    group_id: u64,
    project_id: u64,
    member: Member,
) -> Result<(), String> {
    User::add_project_member(user, group_id, project_id, member).await
}

#[update]
async fn remove_project_member(
    user: Principal,
    group_id: u64,
    project_id: u64,
    member: Principal,
) -> Result<(), String> {
    User::remove_project_member(user, group_id, project_id, member).await
}

#[update]
async fn add_project_canister(
    user: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    User::add_project_canister(user, group_id, project_id, canister).await
}

#[update]
async fn remove_project_canister(
    user: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    User::remove_project_canister(user, group_id, project_id, canister).await
}

#[update]
pub async fn update_project_git_repo_url(
    user: Principal,
    group_id: u64,
    project_id: u64,
    git: String,
) -> Result<(), String> {
    User::update_project_git_repo_url(user, group_id, project_id, git.as_str()).await
}

#[update]
pub async fn update_project_visibility(
    user: Principal,
    group_id: u64,
    project_id: u64,
    visibility: Profile,
) -> Result<(), String> {
    User::update_project_visibility(user, group_id, project_id, visibility).await
}

#[update]
pub async fn update_project_description(
    user: Principal,
    group_id: u64,
    project_id: u64,
    description: String,
) -> Result<(), String> {
    User::update_project_description(user, group_id, project_id, description.as_str()).await
}

#[update]
pub async fn start_project_canister(
    user: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    User::start_project_canister(user, group_id, project_id, canister).await
}

#[update]
pub async fn stop_project_canister(
    user: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    User::stop_project_canister(user, group_id, project_id, canister).await
}

#[update]
pub async fn delete_project_canister(
    user: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    User::delete_project_canister(user, group_id, project_id, canister).await
}

#[update]
pub async fn install_code(
    identity: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
    install_mod: InstallCodeMode,
    wasm: Vec<u8>,
    args: Vec<u8>,
) -> Result<(), String> {
    User::install_code(
        identity,
        group_id,
        project_id,
        canister,
        install_mod,
        wasm,
        args,
    )
    .await
}

#[cfg(test)]
mod operation_test {
    use super::*;
    #[test]
    fn emit_test() {}
}
