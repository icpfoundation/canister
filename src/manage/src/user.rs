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
use std::future::Future;
use std::pin::Pin;
#[macro_use]
use crate::operation;

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

    pub fn get_user_info(&self) -> Result<User, String> {
        // match crate::UserStorage.read().unwrap().get(&identity) {
        //     None => return Err("user does not exist".to_string()),
        //     Some(user) => {
        if caller() != self.identity {
            if let Profile::Private = self.profile {
                return Err("user information is private and cannot be viewed".to_string());
            }
        }

        let mut cp_user = self.clone();
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
        //     }
        // }
    }

    pub fn add_group(&mut self, group: Group) -> Result<(), String> {
        self.identity_check()?;
        self.groups.insert(group.id, group);
        Ok(())
    }

    pub fn remove_group(&mut self, group_id: u64) -> Result<(), String> {
        // match crate::UserStorage.write().unwrap().get_mut(&identity) {
        //     None => return Err("user does not exist".to_string()),
        //     Some(user) => {
        self.identity_check()?;
        self.groups.remove(&group_id);

        Ok(())
        //     }
        // }
    }

    pub fn add_project_relation(
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

    pub fn remove_project_relation(
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

    pub async fn update_member_authority(
        identity: Principal,
        group_id: u64,
        member: Principal,
        auth: Authority,
    ) -> Result<(), String> {
        match crate::UserStorage.write().unwrap().get_mut(&identity) {
            None => return Err("user does not exist".to_string()),
            Some(user) => match user.groups.get_mut(&group_id) {
                None => return Err("Group does not exist".to_string()),
                Some(group) => {
                    group.update_member_authority(member.clone(), auth.clone())?;
                    log!(
                        "update_member_authority",
                        &ic_cdk::caller().to_string(),
                        group_id,
                        &member.to_string(),
                        &auth
                    )()
                    .await;
                    Ok(())
                }
            },
        }
    }

    pub fn add_project(
        &mut self,
        group_id: u64,
        project: Project,
    ) -> Result<Vec<Principal>, String> {
        //let mut user_storage = crate::UserStorage.write().unwrap();
        let mut members: Vec<Principal> = Vec::new();
        let project_id: u64;
        // match user_storage.get_mut(&identity) {
        //     None => return Err("user does not exist".to_string()),

        match self.groups.get_mut(&group_id) {
            None => return Err("Group does not exist".to_string()),
            Some(group) => {
                members = project.members.keys().map(|x| x.clone()).collect();
                project_id = project.id;
                group.add_project(project.clone());
            }
        };
        Ok(members)
    }

    pub fn remove_project(
        &mut self,
        group_id: u64,
        project_id: u64,
    ) -> Result<Vec<Principal>, String> {
        // let mut user_storage = crate::UserStorage.write().unwrap();
        let mut members: Vec<Principal> = Vec::new();

        // match user_storage.get_mut(&identity) {
        //     None => return Err("user does not exist".to_string()),
        match self.groups.get_mut(&group_id) {
            None => return Err("Group does not exist".to_string()),
            Some(group) => {
                if let Some(project) = group.projects.get(&project_id) {
                    members = project.members.keys().map(|x| x.clone()).collect();
                }
                group.remove_project(project_id)?;
            }
        };

        Ok(members)
    }

    pub fn add_group_member(&mut self, group_id: u64, member: Member) -> Result<(), String> {
        self.identity_check()?;
        match self.groups.get_mut(&group_id) {
            None => return Err("Group does not exist".to_string()),
            Some(group) => {
                group.add_member(member.clone())?;
                Ok(())
            }
        }
    }

    pub fn remove_group_member(&mut self, group_id: u64, member: Principal) -> Result<(), String> {
        self.identity_check()?;
        match self.groups.get_mut(&group_id) {
            None => return Err("Group does not exist".to_string()),
            Some(group) => {
                group.remove_member(member);

                Ok(())
            }
        }
    }

    pub fn add_project_member(
        &mut self,
        group_id: u64,
        project_id: u64,
        member: Member,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.add_project_member(project_id, member.clone()),
        }
    }

    pub fn remove_project_member(
        &mut self,
        group_id: u64,
        project_id: u64,
        member: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.remove_project_member(project_id, member),
        }
    }

    pub fn add_project_canister(
        &mut self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.add_project_canister(project_id, canister),
        }
    }

    pub fn remove_project_canister(
        &mut self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.remove_project_canister(project_id, canister),
        }
    }

    pub fn update_project_git_repo_url(
        &mut self,
        group_id: u64,
        project_id: u64,
        git: &str,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.update_git_repo_url(project_id, git),
        }
    }

    pub fn update_project_visibility(
        &mut self,
        group_id: u64,
        project_id: u64,
        visibility: Profile,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.update_visibility(project_id, visibility.clone()),
        }
    }

    pub fn update_project_description(
        &mut self,
        group_id: u64,
        project_id: u64,
        description: &str,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.update_description(project_id, description),
        }
    }

    pub fn get_canister_status(
        &self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<impl Future<Output = Result<CanisterStatusResponse, String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.get_canister_status(project_id, canister),
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

    pub fn stop_project_canister(
        &self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.stop_project_canister(project_id, canister),
        }
    }

    pub fn start_project_canister(
        &self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.start_project_canister(project_id, canister),
        }
    }

    pub fn delete_project_canister(
        &self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.delete_project_canister(project_id, canister),
        }
    }

    pub fn install_code(
        &self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
        install_mod: InstallCodeMode,
        wasm: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.install_code(project_id, canister, install_mod, wasm, args),
        }
    }

    pub fn get_project_info(
        &self,
        group_id: u64,
        project_id: u64,
    ) -> Result<Option<Project>, String> {
        match self.groups.get(&group_id) {
            None => return Ok(None),
            Some(group) => group.get_project_info(project_id).map(|x| x.cloned()),
        }
    }

    pub fn get_group_info(&self, group_id: u64) -> Result<Option<Group>, String> {
        match self.groups.get(&group_id) {
            None => return Ok(None),
            Some(group) => match group.visibility {
                Profile::Public => {
                    return Ok(Some(group.clone()));
                }
                Profile::Private => {
                    let caller = ic_cdk::api::caller();
                    match group.members.get(&caller) {
                        None => {
                            return Err("No permission".to_string());
                        }
                        Some(_) => {
                            return Ok(Some(group.clone()));
                        }
                    }
                }
            },
        }
    }
}
