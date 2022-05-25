use ic_cdk::export::candid::{CandidType, Deserialize};
#[derive(CandidType, Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum Profile {
    Public,
    Private,
}
