use crate::authority::Authority;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct Member {
    pub name: String,
    pub authority: Authority,
    pub identity: Principal,
}

impl Member {
    pub fn new(name: String, authority: Authority, identity: Principal) -> Self {
        Self {
            name: name,
            authority: authority,
            identity: identity,
        }
    }
}
