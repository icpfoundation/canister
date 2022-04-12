use crate::authority::Authority;
use crate::member::Member;
use crate::operation::Operation;
use ic_cdk::api::caller;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use std::collections::HashMap;

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub create_by: Principal,
    pub create_time: u64,
    pub git_repo_url: String,
    pub visibility: Authority,
    pub in_group: u64,
    pub members: HashMap<Principal, Member>,
}

impl Project {
    pub fn new(
        id: u64,
        name: &str,
        description: &str,
        create_by: Principal,
        git: &str,
        visibility: Authority,
        group: u64,
        members: &[Member],
    ) -> Self {
        let create_time = ic_cdk::api::time();
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
        }
    }

    fn identity_check(&self) -> Result<(), String> {
        let operated = self.visibility.clone();

        match self.members.get(&caller()) {
            None => {
                return Err("Not in the group member list".to_string());
            }
            Some(member) => {
                if !Authority::authority_check(member.profile.clone(), operated) {
                    return Err("project does not exist".to_string());
                }
                Ok(())
            }
        }
    }

    pub fn storage(self) -> Result<(), String> {
        let id = self.id;
        if !crate::ProjectStorage.read().unwrap().contains_key(&id) {
            return Err("project iD already exists".to_string());
        }
        crate::ProjectStorage.write().unwrap().insert(id, self);
        Ok(())
    }

    pub fn get_project_by_id(id: u64) -> Option<Project> {
        if !crate::ProjectStorage.read().unwrap().contains_key(&id) {
            return None;
        }
        Some(
            crate::ProjectStorage
                .read()
                .unwrap()
                .get(&id)
                .unwrap()
                .clone(),
        )
    }

    pub fn update_members(&mut self, member: Member, opt: Operation) -> Result<(), String> {
        if let Err(err) =  self.identity_check(){
            return Err(err)
        }
        match opt {
            Operation::Insert => {
                self.members.insert(member.identity, member);
            }
            Operation::Delete => {
                self.members.remove(&member.identity);
            }
        }

        Ok(())
    }
}
