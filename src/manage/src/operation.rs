use ic_cdk::export::candid::{CandidType, Deserialize};

#[derive(CandidType, Debug, Deserialize, Clone)]
pub enum Operation {
    Insert,
    Delete,
}
