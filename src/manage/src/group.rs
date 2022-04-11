use crate::authority::Authority;
use crate::member::Member;
use ic_cdk::api::caller;
use ic_cdk::export::candid::{CandidType, Deserialize};
#[derive(CandidType, Debug, Deserialize)]
pub struct Group {
    id: u64,
    name: String,
    description: String,
    visibility: Authority,
    create_time: u64,
    projects: Vec<u64>,
    members: Vec<Member>,
}

impl Group {
    pub fn new(
        id: u64,
        name: &str,
        description: &str,
        visibility: Authority,
        projects: &[u64],
        members: &[Member],
    ) -> Self {
        let create_time = ic_cdk::api::time();
        Self {
            id: id,
            name: name.to_string(),
            description: description.to_string(),
            visibility: visibility,
            create_time: create_time,
            projects: projects.to_owned(),
            members: members.to_owned(),
        }
    }

    fn identity_check(group_id: u64) -> Result<(), String> {
        let operated = crate::GroupStorage
            .read()
            .unwrap()
            .get(&group_id)
            .unwrap()
            .visibility
            .clone();

        let mut operator: Option<Authority> = None;
        for mem in crate::GroupStorage
            .read()
            .unwrap()
            .get(&group_id)
            .unwrap()
            .members
            .iter()
        {
            if mem.identity == caller() {
                operator = Some(mem.profile.clone());
                break;
            }
        }
        if let None = &operator {
            return Err("Not in the group member list".to_string());
        }
        if !Authority::authority_check(operator.unwrap(), operated) {
            return Err("project does not exist".to_string());
        }
        Ok(())
    }

    pub fn add_project(group_id: u64, project_id: u64) -> Result<(), String> {
        if !crate::GroupStorage.read().unwrap().contains_key(&group_id) {
            return Err("group does not exist".to_string());
        }
        if !crate::ProjectStorage
            .read()
            .unwrap()
            .contains_key(&project_id)
        {
            return Err("project does not exist".to_string());
        }
        if let Err(err) = Self::identity_check(group_id) {
            return Err(err);
        }
        crate::GroupStorage
            .write()
            .unwrap()
            .get_mut(&group_id)
            .unwrap()
            .projects
            .push(project_id);
        Ok(())
    }
}
