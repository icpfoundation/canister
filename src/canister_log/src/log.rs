use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

#[derive(CandidType, Debug, Deserialize, Clone)]
pub enum Action {
    UpdateGroup(u64, String),
    UpdateProject(u64, u64, String),
    UpdateProjectCanister(u64, u64, String),
}

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct Log {
    pub operator: Principal,
    pub create_time: u64,
    pub action: Action,
    pub info: Vec<u8>,
}

impl Log {
    pub fn new(operator: Principal, action: Action, info: Vec<u8>) -> Self {
        let create_time = ic_cdk::api::time();
        Self {
            operator: operator,
            action: action,
            create_time: create_time,
            info: info,
        }
    }
}
