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
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
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

type User_Storage = HashMap<Principal, User>;
thread_local! {
    static USER_STORAGE: RefCell<User_Storage> = RefCell::default();
}

#[init]
async fn init() {}

#[update]
async fn get_canister_status(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<CanisterStatusResponse, String> {
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_canister_status(group_id, project_id, canister),
    })?;
    futures::join!(task).0
}

#[update]
fn add_user(name: String, profile: Profile) -> Result<(), String> {
    let user = User::new(name, profile, ic_cdk::caller());
    USER_STORAGE.with(|user_storage| {
        user_storage.borrow_mut().insert(ic_cdk::caller(), user);
    });
    Ok(())
}

#[update]
async fn add_group(group: Group) -> Result<(), String> {
    USER_STORAGE.with(|user_storage| {
        match user_storage.borrow_mut().get_mut(&ic_cdk::caller()) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_group(group.clone()),
        }
    })?;
    log!(
        "add_group ",
        &ic_cdk::caller().to_string(),
        &ic_cdk::caller().to_string(),
        &group
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_group(group_id: u64) -> Result<(), String> {
    USER_STORAGE.with(|user_storage| {
        match user_storage.borrow_mut().get_mut(&ic_cdk::caller()) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_group(group_id),
        }
    })?;
    log!(
        "remove remove_group",
        &ic_cdk::caller().to_string(),
        &ic_cdk::caller().to_string(),
        &group_id
    )()
    .await;
    Ok(())
}

#[query]
fn get_user_info(ii: Principal) -> Result<User, String> {
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_user_info(),
    })
}

#[update]
async fn add_project(group_id: u64, project: Project) -> Result<(), String> {
    let members = USER_STORAGE.with(|user_storage| {
        match user_storage.borrow_mut().get_mut(&ic_cdk::caller()) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_project(group_id, project.clone()),
        }
    })?;

    USER_STORAGE.with(|user_storage| {
        for i in members {
            match user_storage.borrow_mut().get_mut(&i) {
                None => {
                    return Err("user does not exist".to_string());
                }
                Some(user) => user.add_project_relation(ic_cdk::caller(), group_id, project.id),
            };
        }
        Ok(())
    })?;
    log!("add_project", &ic_cdk::caller().to_string(), &project)().await;
    Ok(())
}

#[update]
async fn remove_project(group_id: u64, project_id: u64) -> Result<(), String> {
    let members = USER_STORAGE.with(|user_storage| {
        match user_storage.borrow_mut().get_mut(&ic_cdk::caller()) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_project(group_id, project_id),
        }
    })?;

    USER_STORAGE.with(|user_storage| {
        for i in members {
            match user_storage.borrow_mut().get_mut(&i) {
                None => {
                    return Err("user does not exist".to_string());
                }
                Some(user) => user.remove_project_relation(ic_cdk::caller(), project_id),
            };
        }
        Ok(())
    })?;

    log!("remove_project", &ic_cdk::caller().to_string(), &project_id)().await;
    Ok(())
}

#[update]
async fn add_group_member(group_id: u64, member: Member) -> Result<(), String> {
    USER_STORAGE.with(|user_storage| {
        match user_storage.borrow_mut().get_mut(&ic_cdk::caller()) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_group_member(group_id, member.clone()),
        }
    })?;

    log!("add_group_member", &ic_cdk::caller().to_string(), &member)().await;
    Ok(())
}

#[update]
async fn remove_group_member(group_id: u64, member: Principal) -> Result<(), String> {
    USER_STORAGE.with(|user_storage| {
        match user_storage.borrow_mut().get_mut(&ic_cdk::caller()) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_group_member(group_id, member),
        }
    })?;

    log!(
        "remove_group_member",
        &ic_cdk::caller().to_string(),
        &member
    )()
    .await;
    Ok(())
}

#[update]
async fn add_project_member(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    member: Member,
) -> Result<(), String> {
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_project_member(group_id, project_id, member.clone()),
        },
    )?;
    USER_STORAGE.with(|user_storage| {
        match user_storage.borrow_mut().get_mut(&member.identity) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_project_relation(ii, group_id, project_id),
        }
    })?;

    log!(
        "add_project_member",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        &member
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_project_member(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    member: Principal,
) -> Result<(), String> {
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_project_member(group_id, project_id, member),
        },
    )?;
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&member) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_project_relation(ii, project_id),
        },
    )?;

    log!(
        "remove_project_member",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        &member
    )()
    .await;
    Ok(())
}

#[update]
async fn add_project_canister(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_project_canister(group_id, project_id, canister),
        },
    )?;

    log!(
        "add_project_canister",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_project_canister(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_project_canister(group_id, project_id, canister),
        },
    )?;

    log!(
        "remove_project_canister",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_project_git_repo_url(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    git: String,
) -> Result<(), String> {
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.update_project_git_repo_url(group_id, project_id, &git),
        },
    )?;
    log!(
        "update_project_git_repo_url",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        git
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_project_visibility(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    visibility: Profile,
) -> Result<(), String> {
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.update_project_visibility(group_id, project_id, visibility.clone()),
        },
    )?;

    log!(
        "update_project_visibility",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        &visibility
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_project_description(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    description: String,
) -> Result<(), String> {
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.update_project_description(group_id, project_id, &description),
        },
    )?;
    log!(
        "update_project_description",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        description
    )()
    .await;
    Ok(())
}

#[update]
pub async fn start_project_canister(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.start_project_canister(group_id, project_id, canister),
    })?;
    futures::join!(task);
    log!(
        "start_project_canister",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
pub async fn stop_project_canister(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.stop_project_canister(group_id, project_id, canister),
    })?;
    futures::join!(task);
    log!(
        "stop_project_canister",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
pub async fn delete_project_canister(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.delete_project_canister(group_id, project_id, canister),
    })?;
    futures::join!(task);
    log!(
        "delete_project_canister",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
pub async fn install_code(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
    install_mod: InstallCodeMode,
    wasm: Vec<u8>,
    args: Vec<u8>,
) -> Result<(), String> {
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.install_code(group_id, project_id, canister, install_mod, wasm, args),
    })?;
    futures::join!(task);
    log!(
        "install_code",
        &ic_cdk::caller().to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[query]
pub fn get_project_info(
    ii: Principal,
    group_id: u64,
    project_id: u64,
) -> Result<Option<Project>, String> {
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_project_info(group_id, project_id),
    })
}

#[query]
pub fn get_group_info(ii: Principal, group_id: u64) -> Result<Option<Group>, String> {
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_group_info(group_id),
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    USER_STORAGE.with(|user_storage| {
        let data_storage: Vec<(Principal, User)> = user_storage
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        ic_cdk::storage::stable_save((data_storage,)).expect("stable_save failed");
    })
}

#[post_upgrade]
fn post_update() {
    let data_storage: (Vec<(Principal, User)>,) =
        ic_cdk::storage::stable_restore().expect("data recovery failed");
    let data_storage: User_Storage = data_storage.0.into_iter().collect();
    USER_STORAGE.with(|user_storage| {
        *user_storage.borrow_mut() = data_storage;
    });
}
