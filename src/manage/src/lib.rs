use ic_cdk_macros::*;
use std::collections::HashMap;
mod authority;
mod constant;
mod group;
mod manage;
mod member;
#[macro_use]
mod operation;
mod project;
mod types;
mod user;
mod util;
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

type User_Storage = HashMap<Principal, User>;

thread_local! {
    static USER_STORAGE: RefCell<User_Storage> = RefCell::default();
    static  OWNER:RefCell<Option<Principal>> = RefCell::default();
}

#[init]
fn init() {
    OWNER.with(|owner| {
        *owner.borrow_mut() = Some(ic_cdk::caller());
    })
}

async fn authority_check(canister: Principal, ii: Principal, sender: Principal) {
    match ManageCanister::get_canister_status(canister, Nat::default()).await {
        Err(err) => ic_cdk::api::trap(&err),
        Ok(status) => match status.0.settings.controllers {
            None => {
                ic_cdk::api::trap("ii is not a canister controller");
            }
            Some(controllers) => {
                if !controllers.contains(&ii) && !controllers.contains(&sender) {
                    ic_cdk::api::trap("no operation permission");
                }
            }
        },
    };
}

#[update]
async fn get_canister_status(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(CanisterStatusResponse, Nat), String> {
    let caller = ic_cdk::api::caller();
    authority_check(canister, ii, caller).await;
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_canister_status(group_id, project_id, canister, caller),
    })?;
    futures::join!(task).0
}

#[update]
fn add_user(name: String, profile: Profile) -> Result<(), String> {
    USER_STORAGE.with(|user_storage| {
        if let Some(_) = user_storage.borrow().get(&ic_cdk::caller()) {
            return Err("User already exists".to_string());
        }
        let user = User::new(name, profile, ic_cdk::caller());
        user_storage.borrow_mut().insert(ic_cdk::caller(), user);
        Ok(())
    })
}

#[update]
async fn add_group(group: Group) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(|user_storage| {
        match user_storage.borrow_mut().get_mut(&ic_cdk::caller()) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_group(group.clone(), caller),
        }
    })?;
    log!(
        &caller.to_string(),
        "add_group",
        &caller.to_string(),
        &group
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_group(group_id: u64) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(|user_storage| {
        match user_storage.borrow_mut().get_mut(&ic_cdk::caller()) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_group(group_id, caller),
        }
    })?;
    log!(
        &caller.to_string(),
        "remove remove_group",
        &caller.to_string(),
        &caller.to_string(),
        &group_id
    )()
    .await;
    Ok(())
}

#[query]
fn get_user_info(ii: Principal) -> Result<User, String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_user_info(caller),
    })
}

#[update]
async fn add_project(group_id: u64, project: Project) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    let members =
        USER_STORAGE.with(
            |user_storage| match user_storage.borrow_mut().get_mut(&caller) {
                None => {
                    return Err("user does not exist".to_string());
                }
                Some(user) => user.add_project(group_id, project.clone(), caller),
            },
        )?;

    USER_STORAGE.with(|user_storage| {
        for i in members {
            match user_storage.borrow_mut().get_mut(&i) {
                None => {
                    return Err("user does not exist".to_string());
                }
                Some(user) => user.add_project_relation(caller, group_id, project.id),
            };
        }
        Ok(())
    })?;
    log!(
        &caller.to_string(),
        "add_project",
        &caller.to_string(),
        &project
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_project(group_id: u64, project_id: u64) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    let members =
        USER_STORAGE.with(
            |user_storage| match user_storage.borrow_mut().get_mut(&caller) {
                None => {
                    return Err("user does not exist".to_string());
                }
                Some(user) => user.remove_project(group_id, project_id, caller),
            },
        )?;

    USER_STORAGE.with(|user_storage| {
        for i in members {
            match user_storage.borrow_mut().get_mut(&i) {
                None => {
                    return Err("user does not exist".to_string());
                }
                Some(user) => user.remove_project_relation(caller, project_id),
            };
        }
        Ok(())
    })?;

    log!(
        &caller.to_string(),
        "remove_project",
        &caller.to_string(),
        &project_id
    )()
    .await;
    Ok(())
}

#[update]
async fn add_group_member(group_id: u64, member: Member) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&caller) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_group_member(group_id, member.clone(), caller),
        },
    )?;

    log!(
        &caller.to_string(),
        "add_group_member",
        &caller.to_string(),
        &member
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_group_member(group_id: u64, member: Principal) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&caller) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_group_member(group_id, member, caller),
        },
    )?;

    log!(
        &caller.to_string(),
        "remove_group_member",
        &caller.to_string(),
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
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_project_member(group_id, project_id, member.clone(), caller),
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
        &caller.to_string(),
        "add_project_member",
        &caller.to_string(),
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
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_project_member(group_id, project_id, member, caller),
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
        &caller.to_string(),
        "remove_project_member",
        &caller.to_string(),
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
    let caller = ic_cdk::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_project_canister(group_id, project_id, canister, caller),
        },
    )?;

    log!(
        &caller.to_string(),
        "add_project_canister",
        &caller.to_string(),
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
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_project_canister(group_id, project_id, canister, caller),
        },
    )?;

    log!(
        &caller.to_string(),
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
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.update_project_git_repo_url(group_id, project_id, &git, caller),
        },
    )?;
    log!(
        &caller.to_string(),
        "update_project_git_repo_url",
        &caller.to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        git
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_canister_cycle_floor(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    floor: Nat,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => {
                user.update_canister_cycle_floor(group_id, project_id, floor.clone(), caller)
            }
        },
    )?;
    log!(
        &caller.to_string(),
        "update_canister_cycle_floor",
        &caller.to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        floor.to_string()
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
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => {
                user.update_project_visibility(group_id, project_id, visibility.clone(), caller)
            }
        },
    )?;

    log!(
        &caller.to_string(),
        "update_project_visibility",
        &caller.to_string(),
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
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => {
                user.update_project_description(group_id, project_id, &description, caller)
            }
        },
    )?;
    log!(
        &caller.to_string(),
        "update_project_description",
        &caller.to_string(),
        &ii.to_string(),
        &group_id,
        &project_id,
        description
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_group_member_authority(
    ii: Principal,
    group_id: u64,
    member: Principal,
    auth: Authority,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => {
                user.update_group_member_authority(group_id, member, auth.clone(), caller)
            }
        },
    )?;
    log!(
        &caller.to_string(),
        "update_group_member_authority",
        &caller.to_string(),
        &ii.to_string(),
        &group_id,
        &member.to_string(),
        auth
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_project_member_authority(
    ii: Principal,
    group_id: u64,
    project_id: u64,
    member: Principal,
    auth: Authority,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&ii) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.update_project_member_authority(
                group_id,
                project_id,
                member,
                auth.clone(),
                caller,
            ),
        },
    )?;
    log!(
        &caller.to_string(),
        "update_project_member_authority",
        &caller.to_string(),
        &ii.to_string(),
        project_id,
        group_id,
        &member.to_string(),
        auth
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
    let caller = ic_cdk::api::caller();
    authority_check(canister, ii, caller).await;
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.start_project_canister(group_id, project_id, canister, caller),
    })?;
    futures::join!(task);
    log!(
        &caller.to_string(),
        "start_project_canister",
        &caller.to_string(),
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
    let caller = ic_cdk::api::caller();
    authority_check(canister, ii, caller).await;
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.stop_project_canister(group_id, project_id, canister, caller),
    })?;
    futures::join!(task);
    log!(
        &caller.to_string(),
        "stop_project_canister",
        &caller.to_string(),
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
    let caller = ic_cdk::api::caller();
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.delete_project_canister(group_id, project_id, canister, caller),
    })?;
    futures::join!(task);
    log!(
        &caller.to_string(),
        "delete_project_canister",
        &caller.to_string(),
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
    let caller = ic_cdk::api::caller();
    authority_check(canister, ii, caller).await;
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.install_code(
            group_id,
            project_id,
            canister,
            install_mod,
            wasm,
            args,
            caller,
        ),
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
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_project_info(group_id, project_id, caller),
    })
}

#[query]
pub fn get_group_info(ii: Principal, group_id: u64) -> Result<Option<Group>, String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&ii) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_group_info(group_id, caller),
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

#[cfg(test)]
mod test_util {
    use super::*;
    #[test]
    fn test_is_controller() {
        let err = String::from("get_canister_status faile: 5: Only the controllers of the canister r7inp-6aaaa-aaaaa-aaabq-cai can control it.\nCanister\'s controllers: rrkah-fqaaa-aaaaa-aaaaq-cai dzhx6-f63tz-aslp6-xxyzd-pknwt-lxpho-q2wsx-pvwwd-v3nq6-75ek5-rqe\nSender\'s ID: r7inp-6aaaa-aaaaa-aaabq-cai");
        let controller = Principal::from_text("r7inp-6aaaa-aaaaa-aaabq-cai").unwrap();
        assert_eq!(util::is_controller(err.clone(), controller), false);

        let controller = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        assert_eq!(util::is_controller(err, controller), true);
    }
}
