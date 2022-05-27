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
use types::{Action, Profile};
use user::User;

type User_Storage = HashMap<Principal, User>;
static mut OWNER: Principal = Principal::from_slice(&[0]);
thread_local! {
    static USER_STORAGE: RefCell<User_Storage> = RefCell::default();

}

#[init]
fn init() {
    unsafe {
        OWNER = ic_cdk::api::caller();
    }
}

#[update]
pub fn update_log_canister(log_canister: Principal) {
    let caller = ic_cdk::api::caller();
    unsafe {
        if OWNER != caller {
            ic_cdk::trap("invalid identity");
        }
        constant::LOG_CANISTER = log_canister;
    }
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
    let caller = ic_cdk::caller();
    USER_STORAGE.with(|user_storage| {
        if let Some(_) = user_storage.borrow().get(&caller) {
            return Err("User already exists".to_string());
        }
        let user = User::new(name, profile, caller);
        user_storage.borrow_mut().insert(caller, user);
        Ok(())
    })
}

#[query]
fn visible_project() -> Vec<Vec<(Principal, u64, Group)>> {
    USER_STORAGE.with(|user_store| {
        user_store
            .borrow()
            .iter()
            .map(|(k, v)| {
                v.groups
                    .iter()
                    .filter(|(group_id, group)| {
                        if let Profile::Public = group.visibility {
                            return true;
                        }
                        false
                    })
                    .map(|(group_id, group)| (*k, *group_id, group.clone()))
                    .collect::<Vec<(Principal, u64, Group)>>()
            })
            .collect::<Vec<Vec<(Principal, u64, Group)>>>()
    })
}
#[update]
async fn add_group(account: Principal, group: Group) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_group(group.clone(), caller),
        },
    )?;
    log!(
        &account.to_string(),
        group.id,
        &caller.to_string(),
        Action::UpdateGroup(group.id, "add_group".to_string()),
        &group
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_group(account: Principal, group_id: u64) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_group(group_id, caller),
        },
    )?;
    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateGroup(group_id, "remove_group".to_string()),
        &group_id
    )()
    .await;
    Ok(())
}

#[query]
fn get_user_info(account: Principal) -> Result<User, String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&account) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_user_info(caller),
    })
}

#[update]
async fn add_project(account: Principal, group_id: u64, project: Project) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    let members =
        USER_STORAGE.with(
            |user_storage| match user_storage.borrow_mut().get_mut(&account) {
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
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProject(group_id, project.id, "add_project".to_string()),
        &project
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_project(account: Principal, group_id: u64, project_id: u64) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    let members =
        USER_STORAGE.with(
            |user_storage| match user_storage.borrow_mut().get_mut(&account) {
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
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProject(group_id, project_id, "remove_project".to_string()),
        &project_id
    )()
    .await;
    Ok(())
}

#[update]
async fn add_group_member(account: Principal, group_id: u64, member: Member) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_group_member(group_id, member.clone(), caller),
        },
    )?;

    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateGroup(group_id, "add_group_member".to_string()),
        &member
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_group_member(
    account: Principal,
    group_id: u64,
    member: Principal,
) -> Result<(), String> {
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
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateGroup(group_id, "remove_group_member".to_string()),
        &member
    )()
    .await;
    Ok(())
}

#[update]
async fn add_project_member(
    account: Principal,
    group_id: u64,
    project_id: u64,
    member: Member,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
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
            Some(user) => user.add_project_relation(account, group_id, project_id),
        }
    })?;

    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProject(group_id, project_id, "add_project_member".to_string()),
        &member
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_project_member(
    account: Principal,
    group_id: u64,
    project_id: u64,
    member: Principal,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
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
            Some(user) => user.remove_project_relation(account, project_id),
        },
    )?;

    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProject(group_id, project_id, "remove_project_member".to_string()),
        &group_id,
        &project_id,
        &member
    )()
    .await;
    Ok(())
}

#[update]
async fn add_project_canister(
    account: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    let caller = ic_cdk::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.add_project_canister(group_id, project_id, canister, caller),
        },
    )?;

    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProjectCanister(group_id, project_id, "add_project_canister".to_string()),
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
async fn remove_project_canister(
    account: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.remove_project_canister(group_id, project_id, canister, caller),
        },
    )?;

    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProjectCanister(group_id, project_id, "remove_project_canister".to_string()),
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_project_git_repo_url(
    account: Principal,
    group_id: u64,
    project_id: u64,
    git: String,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.update_project_git_repo_url(group_id, project_id, &git, caller),
        },
    )?;
    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProject(
            group_id,
            project_id,
            "update_project_git_repo_url".to_string()
        ),
        git
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_canister_cycle_floor(
    account: Principal,
    group_id: u64,
    project_id: u64,
    floor: Nat,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => {
                user.update_canister_cycle_floor(group_id, project_id, floor.clone(), caller)
            }
        },
    )?;
    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProject(
            group_id,
            project_id,
            "update_canister_cycle_floor".to_string()
        ),
        floor.to_string()
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_project_visibility(
    account: Principal,
    group_id: u64,
    project_id: u64,
    visibility: Profile,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => {
                user.update_project_visibility(group_id, project_id, visibility.clone(), caller)
            }
        },
    )?;

    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProject(
            group_id,
            project_id,
            "update_project_visibility".to_string()
        ),
        &visibility
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_project_description(
    account: Principal,
    group_id: u64,
    project_id: u64,
    description: String,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => {
                user.update_project_description(group_id, project_id, &description, caller)
            }
        },
    )?;
    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProject(
            group_id,
            project_id,
            "update_project_description".to_string()
        ),
        description
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_group_member_authority(
    account: Principal,
    group_id: u64,
    member: Principal,
    auth: Authority,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => {
                user.update_group_member_authority(group_id, member, auth.clone(), caller)
            }
        },
    )?;
    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateGroup(group_id, "update_group_member_authority".to_string()),
        &member.to_string(),
        auth
    )()
    .await;
    Ok(())
}

#[update]
pub async fn update_project_member_authority(
    account: Principal,
    group_id: u64,
    project_id: u64,
    member: Principal,
    auth: Authority,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
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
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProject(
            group_id,
            project_id,
            "update_project_member_authority".to_string()
        ),
        &member.to_string(),
        auth
    )()
    .await;
    Ok(())
}

#[update]
async fn update_group_name_and_description_and_visibility(
    account: Principal,
    group_id: u64,
    name: String,
    description: String,
    visibility: Profile,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(
        |user_storage| match user_storage.borrow_mut().get_mut(&account) {
            None => {
                return Err("user does not exist".to_string());
            }
            Some(user) => user.update_group_name_and_description_and_visibility(
                group_id,
                name.clone(),
                description.clone(),
                visibility,
                caller,
            ),
        },
    )?;
    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateGroup(group_id, "update_group_info".to_string()),
        &name,
        &description
    )()
    .await;
    Ok(())
}

#[update]
pub async fn start_project_canister(
    account: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    authority_check(canister, account, caller).await;
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&account) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.start_project_canister(group_id, project_id, canister, caller),
    })?;
    futures::join!(task);
    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProjectCanister(group_id, project_id, "start_project_canister".to_string()),
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
pub async fn stop_project_canister(
    account: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    authority_check(canister, account, caller).await;
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&account) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.stop_project_canister(group_id, project_id, canister, caller),
    })?;
    futures::join!(task);
    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProjectCanister(group_id, project_id, "stop_project_canister".to_string()),
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
pub async fn delete_project_canister(
    account: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    authority_check(canister, account, caller).await;
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&account) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.delete_project_canister(group_id, project_id, canister, caller),
    })?;
    futures::join!(task);
    log!(
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProjectCanister(group_id, project_id, "delete_project_canister".to_string()),
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[update]
pub async fn install_code(
    account: Principal,
    group_id: u64,
    project_id: u64,
    canister: Principal,
    install_mod: InstallCodeMode,
    wasm: Vec<u8>,
    args: Vec<u8>,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();
    authority_check(canister, account, caller).await;
    let task = USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&account) {
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
        &account.to_string(),
        group_id,
        &caller.to_string(),
        Action::UpdateProjectCanister(group_id, project_id, "install_code".to_string()),
        &canister.to_string()
    )()
    .await;
    Ok(())
}

#[query]
pub fn get_project_info(
    account: Principal,
    group_id: u64,
    project_id: u64,
) -> Result<Option<Project>, String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&account) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_project_info(group_id, project_id, caller),
    })
}

#[query]
pub fn get_group_info(account: Principal, group_id: u64) -> Result<Option<Group>, String> {
    let caller = ic_cdk::api::caller();
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&account) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_group_info(group_id, caller),
    })
}

#[query]
pub fn get_group_member_info(
    account: Principal,
    group_id: u64,
    member: Principal,
) -> Result<Member, String> {
    USER_STORAGE.with(|user_storage| match user_storage.borrow().get(&account) {
        None => {
            return Err("user does not exist".to_string());
        }
        Some(user) => user.get_group_member_info(group_id, member),
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
