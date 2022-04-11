use ic_cdk::export::candid::{Deserialize, CandidType};
use ic_cdk::export::Principal;
use crate::authority::Authority;

type Group = String;
type Project = String;

#[derive(CandidType, Debug, Deserialize,Clone)]
pub struct Member{
    pub name:String,
    pub profile:Authority,
    pub identity:Principal,
    pub group:Group,

}