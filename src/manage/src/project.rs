use crate::authority::Authority;
use crate::member::Member;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct Project {
    id: u64,
    name: String,
    description: String,
    create_by: Principal,
    create_time: u64,
    git_repo_url: String,
    visibility: Authority,
    in_group: u64,
    members: Vec<Member>,
}

impl Project {
    pub fn new(
        id: u64,
        name: &str,
        description: &str,
        create_by: Principal,
        git: String,
        visibility: Authority,
        group: u64,
        members: &[Member],
    ) -> Self {
        let create_time = ic_cdk::api::time();
        Self {
            id: id,
            name: name.to_string(),
            description: description.to_string(),
            create_by: create_by,
            create_time: create_time,
            git_repo_url: git,
            visibility: visibility,
            in_group: group,
            members: members.to_owned(),
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

    pub fn update(self) -> Result<(), String> {
        let id = self.id;
        if !crate::ProjectStorage.read().unwrap().contains_key(&id) {
            return Err("Historical project does not exist".to_string());
        }
        crate::ProjectStorage.write().unwrap().insert(id, self);
        Ok(())
    }
}
