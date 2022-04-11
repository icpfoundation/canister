use crate::authority::Authority;
use crate::member::Member;
use crate::operation::Operation;
use crate::project::Project;
use ic_cdk::api::caller;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use std::collections::HashMap;

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct Group {
    id: u64,
    name: String,
    description: String,
    visibility: Authority,
    create_time: u64,
    projects: HashMap<u64, Project>,
    members: HashMap<Principal, Member>,
}

impl Group {
    pub fn new(
        id: u64,
        name: &str,
        description: &str,
        visibility: Authority,
        projects: &[Project],
        members: &[Member],
    ) -> Self {
        let create_time = ic_cdk::api::time();
        let mut member: HashMap<Principal, Member> = HashMap::new();
        let mut project: HashMap<u64, Project> = HashMap::new();
        for i in members.iter() {
            member.insert(i.identity, i.clone());
        }
        for i in projects.iter() {
            project.insert(i.id, i.clone());
        }
        Self {
            id: id,
            name: name.to_string(),
            description: description.to_string(),
            visibility: visibility,
            create_time: create_time,
            projects: project,
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

    pub fn update_project(&mut self, project: Project, opt: Operation) -> Result<(), String> {
        if let Err(err) = self.identity_check() {
            return Err(err);
        }
        match opt {
            Operation::Insert => {
                self.projects.insert(project.id, project);
            }
            Operation::Delete => {
                self.projects.remove(&project.id);
            }
        }

        Ok(())
    }

    pub fn update_members(&mut self, member: Member, opt: Operation) -> Result<(), String> {
        if let Err(err) = self.identity_check() {
            return Err(err);
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

    pub fn storage(self) -> Result<(), String> {
        let id = self.id;
        if !crate::GroupStorage.read().unwrap().contains_key(&id) {
            return Err("project iD already exists".to_string());
        }
        crate::GroupStorage.write().unwrap().insert(id, self);
        Ok(())
    }
}
