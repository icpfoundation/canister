use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct Log {
    pub operator: Principal,
    pub create_time: u64,
    pub info: Vec<u8>,
}

impl Log {
    pub fn new(operator: Principal, info: Vec<u8>) -> Self {
        let create_time = ic_cdk::api::time();
        Self {
            operator: operator,
            create_time: create_time,
            info: info,
        }
    }
}
