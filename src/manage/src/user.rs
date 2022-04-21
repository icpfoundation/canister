use crate::authority::Authority;
use crate::group::Group;
use crate::manage::{CanisterStatusResponse, InstallCodeMode};
use crate::member::Member;
use crate::project::Project;
use crate::types::Profile;
use ic_cdk::api::caller;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use std::collections::HashMap;

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct RelationProject {
    pub group_id: u64,
    pub project_id: u64,
}

impl RelationProject {
    pub fn new(group_id: u64, project_id: u64) -> Self {
        Self {
            group_id: group_id,
            project_id: project_id,
        }
    }
}

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct User {
    pub user_name: String,
    // If it is public, everyone can get the information of the project
    // If it is private, the project can only be accessed by the Creator
    pub profile: Profile,
    // Administrator's IC account
    pub identity: Principal,
    // Users can manage multiple groups, and each group contains multiple items, which is convenient for unified management
    pub groups: HashMap<u64, Group>,

    pub relation_project: HashMap<Principal, Vec<RelationProject>>,
}

impl User {
    pub fn new(user_name: String, profile: Profile, identity: Principal) -> Self {
        Self {
            user_name: user_name,
            profile: profile,
            identity: identity,
            groups: HashMap::new(),
            relation_project: HashMap::new(),
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

    fn add_project_relation(
        &mut self,
        relation_project_user: Principal,
        group_id: u64,
        project_id: u64,
    ) -> Result<(), String> {
        if let Some(relation) = self.relation_project.get_mut(&relation_project_user) {
            relation.push(RelationProject::new(group_id, project_id));
        } else {
            self.relation_project
                .insert(relation_project_user, Vec::new());
            self.relation_project
                .get_mut(&relation_project_user)
                .unwrap()
                .push(RelationProject::new(group_id, project_id));
        }

        Ok(())
    }

    fn remove_project_relation(
        &mut self,
        relation_project_user: Principal,
        project_id: u64,
    ) -> Result<(), String> {
        match self.relation_project.get_mut(&relation_project_user) {
            None => Ok(()),
            Some(projects) => {
                let mut index: Option<usize> = None;
                for (k, v) in projects.iter().enumerate() {
                    if v.project_id == project_id {
                        index = Some(k);
                        break;
                    }
                }
                if let Some(idx) = index {
                    projects.remove(idx);
                }
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
        let mut user_storage = crate::UserStorage.write().unwrap();
        let mut members: Vec<Principal> = Vec::new();
        let project_id: u64;
        match user_storage.get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("Group does not exist".to_string()),
                Some(group) => {
                    members = project.members.keys().map(|x| x.clone()).collect();
                    project_id = project.id;
                    group.add_project(project)?;
                }
            },
        };
        for i in members.iter() {
            if let Some(u) = user_storage.get_mut(i) {
                u.add_project_relation(identity, group_id, project_id);
            }
        }
        Ok(())
    }

    pub fn remove_project(
        identity: Principal,
        group_id: u64,
        project_id: u64,
    ) -> Result<(), String> {
        let mut user_storage = crate::UserStorage.write().unwrap();
        let mut members: Vec<Principal> = Vec::new();

        match user_storage.get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("Group does not exist".to_string()),
                Some(group) => {
                    if let Some(project) = group.projects.get(&project_id) {
                        members = project.members.keys().map(|x| x.clone()).collect();
                    }
                    group.remove_project(project_id)?;
                }
            },
        };

        for i in members.iter() {
            if let Some(u) = user_storage.get_mut(i) {
                u.remove_project_relation(identity, project_id);
            }
        }
        Ok(())
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
        let mut user_storage = crate::UserStorage.write().unwrap();
        let iden: Principal;
        match user_storage.get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => {
                    iden = member.identity.clone();
                    group.add_project_member(project_id, member)?;
                }
            },
        };
        if let Some(u) = user_storage.get_mut(&iden) {
            u.add_project_relation(identity, group_id, project_id)?;
        }
        Ok(())
    }

    pub fn remove_project_member(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        member: Principal,
    ) -> Result<(), String> {
        let mut user_storage = crate::UserStorage.write().unwrap();
        match user_storage.get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("group does not exist".to_string()),

                Some(group) => {
                    group.remove_project_member(project_id, member.clone())?;
                }
            },
        };
        if let Some(u) = user_storage.get_mut(&member) {
            u.remove_project_relation(identity, project_id);
        }
        Ok(())
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

    pub async fn install_code(
        identity: Principal,
        group_id: u64,
        project_id: u64,
        canister: Principal,
        install_mod: InstallCodeMode,
        wasm: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<(), String> {
        match crate::UserStorage.read().unwrap().get(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get(&group_id) {
                None => return Err("group does not exist".to_string()),
                Some(group) => {
                    group
                        .install_code(project_id, canister, install_mod, wasm, args)
                        .await
                }
            },
        }
    }
}
