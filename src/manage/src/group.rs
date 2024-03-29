use crate::authority::Authority;
use crate::manage::{CanisterSettings, CanisterStatusResponse, InstallCodeMode, ManageCanister};
use crate::member::Member;
use crate::project::Project;
use crate::types::Profile;
use candid::CandidType;
use ic_cdk::api::caller;
use ic_cdk::export::candid::Deserialize;
use ic_cdk::export::candid::Nat;
use ic_cdk::export::Principal;
use std::collections::HashMap;
use std::future::Future;
#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct Group {
    pub id: u64,
    pub create_time: u64,
    pub visibility: Profile,
    pub name: String,
    pub description: String,
    pub projects: HashMap<u64, Project>,
    pub members: HashMap<Principal, Member>,
    pub url: String,
}

impl Group {
    pub fn new(
        id: u64,
        create_time: u64,
        visibility: Profile,
        name: &str,
        description: &str,
        projects: Vec<Project>,
        members: Vec<Member>,
        url: String,
    ) -> Self {
        let member = members
            .iter()
            .map(|v| v.identity)
            .collect::<Vec<Principal>>();
        let member: HashMap<Principal, Member> =
            member.into_iter().zip(members.into_iter()).collect();

        let project_id = projects.iter().map(|v| v.id).collect::<Vec<u64>>();
        let project: HashMap<u64, Project> =
            project_id.into_iter().zip(projects.into_iter()).collect();

        Self {
            id: id,
            create_time: create_time,
            name: name.to_string(),
            description: description.to_string(),
            visibility: visibility,
            projects: project,
            members: member,
            url: url,
        }
    }

    pub fn identity_check(&self, opt: Authority, sender: Principal) -> Result<(), String> {
        match self.members.get(&sender) {
            None => {
                return Err("not in the group member list".to_string());
            }
            Some(member) => {
                if !Authority::authority_check(member.authority, opt) {
                    return Err(format!("permission verification failed: user permissions: {:?},opt permissions: {:?}",member.authority,opt) );
                }
                if let Some(expir) = member.expiration_time {
                    if expir < ic_cdk::api::time() {
                        return Err("Identity expiration".to_string());
                    }
                }
                Ok(())
            }
        }
    }

    pub fn add_project(&mut self, project: Project, sender: Principal) -> Result<(), String> {
        self.identity_check(Authority::Operational, sender)?;
        if self.projects.contains_key(&project.id) {
            return Err("project id already exists".to_string());
        }
        self.projects.insert(project.id, project);
        Ok(())
    }

    pub fn remove_project(&mut self, project_id: u64, sender: Principal) -> Result<(), String> {
        self.identity_check(Authority::Operational, sender)?;
        self.projects.remove(&project_id);

        Ok(())
    }

    pub fn add_member(&mut self, member: Member) -> Result<(), String> {
        self.members.insert(member.identity, member);
        Ok(())
    }

    pub fn remove_member(&mut self, member: Principal) -> Result<(), String> {
        self.members.remove(&member);
        Ok(())
    }

    pub fn get_group_projects_info(
        &self,
        sender: Principal,
    ) -> Result<Option<Vec<Project>>, String> {
        self.identity_check(Authority::Read, sender)?;
        return Ok(Some(self.projects.values().map(|i| i.clone()).collect()));
    }

    pub fn update_member_authority(
        &mut self,
        member: Principal,
        authority: Authority,
    ) -> Result<(), String> {
        match self.members.get_mut(&member) {
            None => return Err("member information not found".to_string()),
            Some(data) => {
                data.authority = authority;
                return Ok(());
            }
        }
    }

    pub fn update_project_member_authority(
        &mut self,
        project_id: u64,
        member: Principal,
        authority: Authority,
        sender: Principal,
    ) -> Result<(), String> {
        self.identity_check(Authority::Operational, sender)?;
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.update_member_authority(member, authority),
        }
    }

    pub fn update_git_repo_url(
        &mut self,
        project_id: u64,
        git: &str,
        sender: Principal,
    ) -> Result<(), String> {
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.update_git_repo_url(git, sender),
        }
    }

    pub fn update_canister_cycle_floor(
        &mut self,
        project_id: u64,
        floor: Nat,
        sender: Principal,
    ) -> Result<(), String> {
        self.identity_check(Authority::Write, sender)?;
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.update_canister_cycle_floor(floor, sender),
        }
    }

    pub fn update_visibility(
        &mut self,
        project_id: u64,
        visibility: Profile,
        sender: Principal,
    ) -> Result<(), String> {
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.update_visibility(visibility, sender),
        }
    }

    pub fn update_description(
        &mut self,
        project_id: u64,
        description: &str,
        sender: Principal,
    ) -> Result<(), String> {
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.update_description(description, sender),
        }
    }

    pub fn add_project_member(
        &mut self,
        project_id: u64,
        member: Member,
        sender: Principal,
    ) -> Result<(), String> {
        self.identity_check(Authority::Operational, sender)?;
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.add_member(member),
        }
    }

    pub fn remove_project_member(
        &mut self,
        project_id: u64,
        member: Principal,
        sender: Principal,
    ) -> Result<(), String> {
        if member != sender {
            self.identity_check(Authority::Operational, sender)?;
        }
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.remove_member(member),
        }
    }

    pub fn add_project_canister(
        &mut self,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<(), String> {
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.add_canister(canister, sender),
        }
    }

    pub fn remove_project_canister(
        &mut self,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<(), String> {
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.remove_canister(canister, sender),
        }
    }

    pub fn get_canister_status(
        &self,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(CanisterStatusResponse, Nat), String>>, String> {
        match self.projects.get(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => {
                project.get_canister_status(canister, sender, Profile::Private == self.visibility)
            }
        }
    }

    pub async fn set_canister_controller(
        &self,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<(), String> {
        match self.projects.get(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.set_canister_controller(canister, sender).await,
        }
    }

    pub fn stop_project_canister(
        &self,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.projects.get(&project_id) {
            None => Err("Project does not exist....".to_string()),
            Some(project) => project.stop_canister(canister, sender),
        }
    }

    pub fn start_project_canister(
        &self,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.projects.get(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.start_canister(canister, sender),
        }
    }

    pub fn delete_project_canister(
        &self,
        project_id: u64,
        canister: Principal,
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.projects.get(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.delete_canister(canister, sender),
        }
    }

    pub fn install_code(
        &self,
        project_id: u64,
        canister: Principal,
        install_mod: InstallCodeMode,
        wasm: Vec<u8>,
        args: Vec<u8>,
        sender: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        match self.projects.get(&project_id) {
            None => Err("Project does not exist".to_string()),
            Some(project) => project.install_code(canister, install_mod, wasm, args, sender),
        }
    }

    pub fn get_project_info(
        &self,
        project_id: u64,
        sender: Principal,
    ) -> Result<Option<&Project>, String> {
        match self.visibility {
            Profile::Public => match self.projects.get(&project_id) {
                None => Ok(None),
                Some(project) => match project.visibility {
                    Profile::Public => Ok(Some(project)),
                    Profile::Private => {
                        if let None = project.members.get(&sender) {
                            return Err("No permission".to_string());
                        }
                        Ok(Some(project))
                    }
                },
            },
            Profile::Private => {
                if let None = self.members.get(&sender) {
                    match self.projects.get(&project_id) {
                        None => return Ok(None),
                        Some(project) => {
                            if let None = project.members.get(&sender) {
                                return Err("No permission".to_string());
                            }
                            return Ok(Some(project));
                        }
                    }
                };
                Ok(self.projects.get(&project_id))
            }
        }
    }

    pub fn update_basic_information(
        &mut self,
        name: String,
        description: String,
        visibility: Profile,
        url: String,
        sender: Principal,
    ) -> Result<(), String> {
        self.identity_check(Authority::Write, sender)?;
        self.name = name;
        self.description = description;
        self.visibility = visibility;
        self.url = url;
        Ok(())
    }

    pub fn update_project_basic_information(
        &mut self,
        project_id: u64,
        name: String,
        description: String,
        visibility: Profile,
        git: String,
        canister_cycle_floor: Nat,
        canisters: &[Principal],
        sender: Principal,
    ) -> Result<(), String> {
        let mut check = true;
        if let Ok(()) = self.identity_check(Authority::Write, sender) {
            check = false;
        }
        match self.projects.get_mut(&project_id) {
            None => Err("Project does not exist".to_string()),

            Some(project) => project.update_basic_information(
                name,
                description,
                visibility,
                git,
                canister_cycle_floor,
                canisters,
                sender,
                check,
            ),
        }
    }
}
