use crate::authority::Authority;
use crate::manage::{CanisterSettings, CanisterStatusResponse, InstallCodeMode, ManageCanister};
use crate::member::Member;
use crate::types::Profile;
use ic_cdk::api::caller;
use ic_cdk::export::candid::Nat;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use std::collections::HashMap;
use std::future::Future;
#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct Project {
    pub id: u64,
    pub create_time: u64,
    pub in_group: u64,
    pub visibility: Profile,
    pub create_by: Principal,
    pub name: String,
    pub description: String,
    pub git_repo_url: String,
    pub members: HashMap<Principal, Member>,
    pub canister_cycle_floor: Nat,
    pub canisters: Vec<Principal>,
}

impl Project {
    pub fn new(
        id: u64,
        create_time: u64,
        group: u64,
        name: &str,
        description: &str,
        create_by: Principal,
        git: &str,
        visibility: Profile,
        members: &[Member],
        canister_cycle_floor: Nat,
        canisters: &[Principal],
    ) -> Self {
        let mut member: HashMap<Principal, Member> = HashMap::new();
        for i in members.iter() {
            member.insert(i.identity, i.clone());
        }
        Self {
            id: id,
            name: name.to_string(),
            description: description.to_string(),
            create_by: create_by,
            create_time: create_time,
            git_repo_url: git.to_string(),
            visibility: visibility,
            in_group: group,
            members: member,
            canister_cycle_floor: canister_cycle_floor,
            canisters: canisters.to_owned(),
        }
    }

    fn identity_check(&self, opt: Authority) -> Result<(), String> {
        match self.members.get(&caller()) {
            None => {
                return Err("not in the group member list".to_string());
            }
            Some(member) => {
                if !Authority::authority_check(member.authority.clone(), opt.clone()) {
                    return Err(format!("project permission verification failed: user permissions: {:?},opt permissions: {:?}",member.authority.clone(),opt));
                }
                Ok(())
            }
        }
    }

    pub fn add_member(&mut self, member: Member) -> Result<(), String> {
        self.members.insert(member.identity, member);
        Ok(())
    }

    pub fn remove_member(&mut self, member: Principal) -> Result<(), String> {
        self.members.remove(&member);
        Ok(())
    }

    pub fn update_git_repo_url(&mut self, git: &str) -> Result<(), String> {
        self.identity_check(Authority::Write)?;
        self.git_repo_url = git.to_string();
        Ok(())
    }
    pub fn update_canister_cycle_floor(&mut self, floor: Nat) -> Result<(), String> {
        self.identity_check(Authority::Write)?;
        self.canister_cycle_floor = floor;
        Ok(())
    }

    pub fn update_visibility(&mut self, visibility: Profile) -> Result<(), String> {
        self.identity_check(Authority::Write)?;
        self.visibility = visibility;
        Ok(())
    }

    pub fn update_description(&mut self, description: &str) -> Result<(), String> {
        self.identity_check(Authority::Write)?;
        self.description = description.to_string();
        Ok(())
    }

    pub fn add_canister(&mut self, canister: Principal) -> Result<(), String> {
        if self.canisters.contains(&canister) {
            return Err("canisters already exist".to_string());
        }
        self.canisters.push(canister);
        Ok(())
    }

    pub fn remove_canister(&mut self, canister: Principal) -> Result<(), String> {
        if self.canisters.contains(&canister) {
            return Err("canisters do not exist".to_string());
        }
        self.canisters.retain(|&x| x != canister);
        Ok(())
    }

    pub fn update_member_authority(
        &mut self,
        member: Principal,
        authority: Authority,
    ) -> Result<(), String> {
        match self.members.get_mut(&member) {
            None => Err("member is not in the project".to_string()),
            Some(member) => {
                member.authority = authority;
                return Ok(());
            }
        }
    }

    pub fn get_canister_status(
        &self,
        canister: Principal,
    ) -> Result<impl Future<Output = Result<(CanisterStatusResponse, Nat), String>>, String> {
        if self.canisters.contains(&canister) {
            self.identity_check(Authority::Read)?;
            let canister_cycle_floor = self.canister_cycle_floor.clone();
            return Ok(async move {
                ManageCanister::get_canister_status(canister, canister_cycle_floor).await
            });
        }
        return Err("canisters do not exist in the project".to_string());
    }

    pub async fn set_canister_controller(&self, canister: Principal) -> Result<(), String> {
        if self.canisters.contains(&canister) {
            self.identity_check(Authority::Operational)?;
            let controllers: Option<Vec<Principal>> = Some(vec![ic_cdk::api::caller()]);
            let compute_allocation: Nat = "0".parse().unwrap();
            let memory_allocation: Nat = "0".parse().unwrap();
            let freezing_threshold: Nat = "2_592_000".parse().unwrap();
            let canister_settings = CanisterSettings::new(
                controllers,
                Some(compute_allocation),
                Some(memory_allocation),
                Some(freezing_threshold),
            );
            let mange_canister = ManageCanister::new(canister, canister_settings);
            return mange_canister.set_controller().await;
        }
        return Err("canisters do not exist in the project".to_string());
    }

    pub fn stop_canister(
        &self,
        canister: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        if self.canisters.contains(&canister) {
            self.identity_check(Authority::Operational)?;
            return Ok(async move { ManageCanister::stop_canister(canister).await });
        }
        return Err("canisters do not exist in the project".to_string());
    }

    pub fn start_canister(
        &self,
        canister: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        if self.canisters.contains(&canister) {
            self.identity_check(Authority::Operational)?;
            return Ok(async move { ManageCanister::start_canister(canister).await });
        }
        return Err("canisters do not exist in the project".to_string());
    }

    pub fn delete_canister(
        &self,
        canister: Principal,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        if self.canisters.contains(&canister) {
            self.identity_check(Authority::Operational)?;
            return Ok(async move { ManageCanister::delete_canister(canister).await });
        }
        return Err("canisters do not exist in the project".to_string());
    }

    pub fn install_code(
        &self,
        canister: Principal,
        install_mod: InstallCodeMode,
        wasm: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<impl Future<Output = Result<(), String>>, String> {
        if self.canisters.contains(&canister) {
            self.identity_check(Authority::Operational)?;
            return Ok(async move {
                ManageCanister::install_code(canister, install_mod, wasm, args).await
            });
        }
        return Err("canisters do not exist in the project".to_string());
    }
}
