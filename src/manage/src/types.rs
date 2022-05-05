use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(CandidType, Debug, Deserialize, Clone)]
pub enum Profile {
    Public,
    Private,
}
