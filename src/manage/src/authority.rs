use ic_cdk::export::candid::{CandidType, Deserialize};
use serde::Serialize;

// Project permission management is similar to Linux file operation permission
#[derive(CandidType, Debug, Deserialize, Clone, Serialize, Copy)]
pub enum Authority {
    // You can read the basic information of groups or projects, but you cannot modify them
    Read,
    // You can read and modify the basic information of the project,
    // but you can't operate more core functions, such as deleting canisters
    Write,
    // You can do anything
    Operational,
}

impl Authority {
    pub fn authority_check(operator: Authority, opt: Authority) -> bool {
        let opt = match opt {
            Self::Read => 1u8,
            Self::Write => 2u8,
            Self::Operational => 4u8,
        };

        let operator = match operator {
            Self::Read => 1u8,
            Self::Write => 2u8,
            Self::Operational => 4u8,
        };

        if operator >= opt {
            return true;
        }
        return false;
    }
}
