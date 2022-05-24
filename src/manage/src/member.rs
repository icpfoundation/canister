use crate::authority::Authority;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct Member {
    pub name: String,
    pub authority: Authority,
    pub identity: Principal,
    pub join_time: u64,
}

impl Member {
    pub fn new(name: String, authority: Authority, identity: Principal, join_time: u64) -> Self {
        Self {
            name: name,
            authority: authority,
            identity: identity,
            join_time: join_time,
        }
    }
}
