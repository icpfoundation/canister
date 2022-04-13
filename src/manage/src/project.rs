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
    pub create_time: u64,
    pub in_group: u64,
    pub visibility: Authority,
    pub create_by: Principal,
    pub name: String,
    pub description: String,
    pub git_repo_url: String,
    pub members: HashMap<Principal, Member>,
}

impl Project {
    pub fn new(
        id: u64,
        create_time:u64,
        group: u64,
        name: &str,
        description: &str,
        create_by: Principal,
        git: &str,
        visibility: Authority,
        members: &[Member],
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
        }
    }

    fn identity_check(&self, opt: Authority) -> Result<(), String> {
        let operated = self.visibility.clone();

        match self.members.get(&caller()) {
            None => {
                return Err("Not in the group member list".to_string());
            }
            Some(member) => {
                if !Authority::authority_check(member.profile.clone(), operated, opt) {
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



    pub fn add_member(&mut self, member: Member) -> Result<(), String> {
        if let Err(err) = self.identity_check(Authority::Write) {
            return Err(err);
        }

        self.members.insert(member.identity, member);

        Ok(())
    }

    pub fn remove_member(&mut self, member: Principal) -> Result<(), String> {
        if let Err(err) = self.identity_check(Authority::Write) {
            return Err(err);
        }

        self.members.remove(&member);
        Ok(())
    }
}
