use crate::authority::Authority;
use crate::group::Group;
use crate::manage::CanisterStatusResponse;
use crate::member::Member;
use crate::operation::Operation;
use crate::project::Project;
use crate::types::Profile;
use ic_cdk::api::caller;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use std::collections::HashMap;

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct User {
    pub user_name: String,
    pub profile: Profile,
    pub identity: Principal,
    pub groups: HashMap<u64, Group>,
}

impl User {
    pub fn new(user_name: String, profile: Profile, identity: Principal) -> Self {
        Self {
            user_name: user_name,
            profile: profile,
            identity: identity,
            groups: HashMap::new(),
        }
    }

    fn identity_check(&self) -> Result<(), String> {
        if self.identity == caller() {
            return Ok(());
        }
        return Err("no permission".to_string());
    }

    pub fn get_user_info(identity: Principal) -> Result<User, String> {
        match crate::UserStorage.read().unwrap().get(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => {
                if caller() != identity {
                    if let Profile::Private = user.profile {
                        return Err("user information is private and cannot be viewed".to_string());
                    }
                }

                let mut cp_user = user.clone();
                let publick_group: HashMap<u64, Group> = cp_user
                    .groups
                    .into_iter()
                    .filter(|(k, v)| {
                        if let Profile::Public = v.visibility {
                            return true;
                        };
                        return false;
                    })
                    .collect();

                cp_user.groups = publick_group;
                Ok(cp_user)
            }
        }
    }

    pub fn add_group(identity: Principal, group: Group) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => {
                user.identity_check()?;
                user.groups.insert(group.id, group);
                Ok(())
            }
        }
    }

    pub fn remove_group(identity: Principal, group_id: u64) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => {
                user.identity_check()?;
                user.groups.remove(&group_id);
                Ok(())
            }
        }
    }

    pub fn update_member_authority(
        identity: Principal,
        group_id: u64,
        member: Principal,
        auth: Authority,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("Group does not exist".to_string()),
                Some(group) => group.update_member_authority(member, auth),
            },
        }
    }

    pub fn add_project(identity: Principal, group_id: u64, project: Project) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("Group does not exist".to_string()),
                Some(group) => group.add_project(project),
            },
        }
    }

    pub fn remove_project(
        identity: Principal,
        group_id: u64,
        project_id: u64,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("Group does not exist".to_string()),
                Some(group) => group.remove_project(project_id),
            },
        }
    }

    pub fn add_group_member(
        identity: Principal,
        group_id: u64,
        member: Member,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => {
                user.identity_check()?;
                match user.groups.get_mut(&group_id) {
                    None => return Err("Group does not exist".to_string()),
                    Some(group) => group.add_member(member),
                }
            }
        }
    }

    pub fn remove_group_member(
        identity: Principal,
        group_id: u64,
        member: Principal,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => {
                user.identity_check()?;
                match user.groups.get_mut(&group_id) {
                    None => return Err("Group does not exist".to_string()),
                    Some(group) => group.remove_member(member),
                }
            }
        }
    }

    pub fn add_project_member(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        member: Member,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.add_project_member(project_id, member),
            },
        }
    }

    pub fn remove_project_member(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        member: Principal,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.remove_project_member(project_id, member),
            },
        }
    }

    pub fn add_project_canister(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.add_project_canister(project_id, canister),
            },
        }
    }

    pub fn remove_project_canister(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.remove_project_canister(project_id, canister),
            },
        }
    }

    pub fn update_project_git_repo_url(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        git: &str,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.update_git_repo_url(project_id, git),
            },
        }
    }

    pub fn update_project_visibility(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        visibility: Profile,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.update_visibility(project_id, visibility),
            },
        }
    }

    pub fn update_project_description(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        description: &str,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.update_description(project_id, description),
            },
        }
    }

    pub async fn get_canister_status(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<CanisterStatusResponse, String> {
        match crate::UserStorage.read().unwrap().get(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.get_canister_status(project_id, canister).await,
            },
        }
    }

    pub fn storage(self) -> Result<(), String> {
        if crate::UserStorage
            .read()
            .unwrap()
            .contains_key(&self.identity)
        {
            return Err("user already exists".to_string());
        }
        crate::UserStorage
            .write()
            .unwrap()
            .insert(self.identity, self);
        Ok(())
    }

    pub async fn stop_project_canister(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<(), String> {
        match crate::UserStorage.read().unwrap().get(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.stop_project_canister(project_id, canister).await,
            },
        }
    }

    pub async fn start_project_canister(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<(), String> {
        match crate::UserStorage.read().unwrap().get(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.start_project_canister(project_id, canister).await,
            },
        }
    }

    pub async fn delete_project_canister(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<(), String> {
        match crate::UserStorage.read().unwrap().get(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => group.delete_project_canister(project_id, canister).await,
            },
        }
    }
}
