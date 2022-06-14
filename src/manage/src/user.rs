use crate::authority::Authority;
use crate::group::Group;
use crate::manage::{CanisterStatusResponse, InstallCodeMode};
use crate::member::Member;
use crate::project::Project;
use crate::types::Profile;
use ic_cdk::api::caller;
use ic_cdk::export::candid::Nat;
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

    pub create_time: u64,
}

impl User {
    pub fn new(user_name: String, profile: Profile, identity: Principal, create_time: u64) -> Self {
        Self {
            user_name: user_name,
            profile: profile,
            identity: identity,
            groups: HashMap::new(),
            relation_project: HashMap::new(),
            create_time: create_time,
        }
    }

    fn identity_check(&self, sender: Principal) -> Result<(), String> {
        if self.identity == sender {
            return Ok(());
        }
        return Err("no permission".to_string());
    }

    pub fn get_user_info(&self, sender: Principal) -> Result<User, String> {
        if sender != self.identity {
            if let Profile::Private = self.profile {
                return Err("user information is private and cannot be viewed".to_string());
            }

            let mut cp_user = self.clone();
            let public_group: HashMap<u64, Group> = cp_user
                .groups
                .into_iter()
                .filter(|(k, v)| {
                    if let Profile::Public = v.visibility {
                        return true;
                    };
                    if let Some(mem) = v.members.get(&sender) {
                        if let Some(expir) = mem.expiration_time {
                            if expir < ic_cdk::api::time() {
                                return false;
                            }
                        }
                        return true;
                    }
                    return false;
                })
                .collect();

            cp_user.groups = public_group;
            return Ok(cp_user);
        } else {
            return Ok(self.clone());
        }
    }

    pub fn add_group(&mut self, group: Group, sender: Principal) -> Result<(), String> {
        self.identity_check(sender)?;
        if self.groups.contains_key(&group.id) {
            return Err("group id already exists".to_string());
        }
        self.groups.insert(group.id, group);
        Ok(())
    }

    pub fn remove_group(&mut self, group_id: u64, sender: Principal) -> Result<(), String> {
        self.identity_check(sender)?;
        self.groups.remove(&group_id);
        Ok(())
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

    pub fn update_group_member_authority(
        &mut self,
        group_id: u64,
        member: Principal,
        auth: Authority,
        sender: Principal,
    ) -> Result<(), String> {
        self.identity_check(sender)?;
        match self.groups.get_mut(&group_id) {
            None => return Err("Group does not exist".to_string()),
            Some(group) => {
                group.update_member_authority(member, auth)?;
                Ok(())
            }
        }
    }

    pub fn update_project_member_authority(
        &mut self,
        group_id: u64,
        project_id: u64,
        member: Principal,
        auth: Authority,
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("Group does not exist".to_string()),
            Some(group) => {
                group.update_project_member_authority(project_id, member, auth, sender)?;
                Ok(())
            }
        }
    }

    pub fn add_project(
        &mut self,
        group_id: u64,
        project: Project,
        sender: Principal,
    ) -> Result<Vec<Principal>, String> {
        let mut members: Vec<Principal> = Vec::new();
        let project_id: u64;

        match self.groups.get_mut(&group_id) {
            None => return Err("Group does not exist".to_string()),
            Some(group) => {
                members = project.members.keys().map(|x| *x).collect();
                project_id = project.id;
                group.add_project(project.clone(), sender)?;
            }
        };
        Ok(members)
    }

    pub fn remove_project(
        &mut self,
        group_id: u64,
        project_id: u64,
        sender: Principal,
    ) -> Result<Vec<Principal>, String> {
        let mut members: Vec<Principal> = Vec::new();

        match self.groups.get_mut(&group_id) {
            None => return Err("Group does not exist".to_string()),
            Some(group) => {
                if let Some(project) = group.projects.get(&project_id) {
                    members = project.members.keys().map(|x| *x).collect();
                }
                if self.identity == sender {
                    group.projects.remove(&project_id);
                } else {
                    group.remove_project(project_id, sender)?;
                }
            }
        };

        Ok(members)
    }

    pub fn add_group_member(
        &mut self,
        group_id: u64,
        member: Member,
        sender: Principal,
    ) -> Result<(), String> {
        self.identity_check(sender)?;
        match self.groups.get_mut(&group_id) {
            None => return Err("Group does not exist".to_string()),
            Some(group) => {
                group.add_member(member)?;
                Ok(())
            }
        }
    }

    pub fn remove_group_member(
        &mut self,
        group_id: u64,
        member: Principal,
        sender: Principal,
    ) -> Result<(), String> {
        if member != sender {
            self.identity_check(sender)?;
        }
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
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.add_project_member(project_id, member, sender),
        }
    }

    pub fn remove_project_member(
        &mut self,
        group_id: u64,
        project_id: u64,
        member: Principal,
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.remove_project_member(project_id, member, sender),
        }
    }

    pub fn add_project_canister(
        &mut self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.add_project_canister(project_id, canister, sender),
        }
    }

    pub fn remove_project_canister(
        &mut self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.remove_project_canister(project_id, canister, sender),
        }
    }

    pub fn update_project_git_repo_url(
        &mut self,
        group_id: u64,
        project_id: u64,
        git: &str,
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.update_git_repo_url(project_id, git, sender),
        }
    }

    pub fn update_canister_cycle_floor(
        &mut self,
        group_id: u64,
        project_id: u64,
        floor: Nat,
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.update_canister_cycle_floor(project_id, floor, sender),
        }
    }

    pub fn update_project_visibility(
        &mut self,
        group_id: u64,
        project_id: u64,
        visibility: Profile,
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.update_visibility(project_id, visibility, sender),
        }
    }

    pub fn update_project_description(
        &mut self,
        group_id: u64,
        project_id: u64,
        description: &str,
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.update_description(project_id, description, sender),
        }
    }

    pub fn get_canister_status(
        &self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(CanisterStatusResponse, Nat), String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.get_canister_status(project_id, canister, sender),
        }
    }

    pub fn stop_project_canister(
        &self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.stop_project_canister(project_id, canister, sender),
        }
    }

    pub fn start_project_canister(
        &self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.start_project_canister(project_id, canister, sender),
        }
    }

    pub fn delete_project_canister(
        &self,
        group_id: u64,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.delete_project_canister(project_id, canister, sender),
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
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => {
                group.install_code(project_id, canister, install_mod, wasm, args, sender)
            }
        }
    }

    pub fn get_project_info(
        &self,
        group_id: u64,
        project_id: u64,
        sender: Principal,
    ) -> Result<Option<Project>, String> {
        match self.groups.get(&group_id) {
            None => return Ok(None),
            Some(group) => {
                if self.identity == sender {
                    return Ok(group.projects.get(&project_id).cloned());
                } else {
                    group
                        .get_project_info(project_id, sender)
                        .map(|x| x.cloned())
                }
            }
        }
    }

    pub fn get_group_info(
        &self,
        group_id: u64,
        sender: Principal,
    ) -> Result<Option<Group>, String> {
        match self.groups.get(&group_id) {
            None => return Ok(None),
            Some(group) => match group.visibility {
                Profile::Public => {
                    return Ok(Some(group.clone()));
                }
                Profile::Private => match group.members.get(&sender) {
                    None => {
                        if self.identity == sender {
                            return Ok(Some(group.clone()));
                        }
                        return Err("No permission".to_string());
                    }
                    Some(_) => {
                        return Ok(Some(group.clone()));
                    }
                },
            },
        }
    }

    pub fn get_group_member_info(
        &self,
        group_id: u64,
        member: Principal,
    ) -> Result<Member, String> {
        match self.groups.get(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => {
                let res = group
                    .members
                    .get(&member)
                    .ok_or("Member not found".to_string());
                Ok(res.unwrap().clone())
            }
        }
    }

    pub fn update_group_basic_information(
        &mut self,
        group_id: u64,
        name: String,
        description: String,
        visibility: Profile,
        url: String,
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => {
                group.update_basic_information(name, description, visibility, url, sender)
            }
        }
    }

    pub fn update_project_basic_information(
        &mut self,
        group_id: u64,
        project_id: u64,
        name: String,
        description: String,
        visibility: Profile,
        git: String,
        canister_cycle_floor: Nat,
        canisters: &[Principal],
        sender: Principal,
    ) -> Result<(), String> {
        match self.groups.get_mut(&group_id) {
            None => return Err("group does not exist".to_string()),
            Some(group) => group.update_project_basic_information(
                project_id,
                name,
                description,
                visibility,
                git,
                canister_cycle_floor,
                canisters,
                sender,
            ),
        }
    }
}
