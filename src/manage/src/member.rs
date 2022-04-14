use ic_cdk::export::candid::{Deserialize, CandidType};
use ic_cdk::export::Principal;
use crate::authority::Authority;

#[derive(CandidType, Debug, Deserialize,Clone)]
pub struct Member{
    pub name:String,
    pub authority:Authority,
    pub identity:Principal,
}

impl Member{
    pub fn new(name:String,authority:Authority,identity:Principal) -> Self{
        Self{
            name:name,
            authority:authority,
            identity:identity,
        }
    }
}