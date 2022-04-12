use ic_cdk::export::candid::{CandidType, Deserialize};
use serde::Serialize;

// Project permission management is similar to Linux file operation permission
#[derive(CandidType, Debug, Deserialize, Clone,Serialize)]
pub enum Authority {
    // You can read the basic information of groups or projects, but you cannot modify them
    ReadOnly,
    // You can read and modify the basic information of the project,
    // but you can't operate more core functions, such as deleting canisters
    ReadAndWrite,
    // You can do anything
    Operational,
}


impl Authority {
    pub fn authority_check(operator: Authority, operated: Authority) -> bool {
        let operator = match operator {
            Self::ReadOnly => 1u8,
            Self::ReadAndWrite => 2u8,
            Self::Operational => 4u8,
        };

        let operated = match operated {
            Self::ReadOnly => 1u8,
            Self::ReadAndWrite => 2u8,
            Self::Operational => 4u8,
        };

        if operator >= operated {
            return true;
        }
        return false;
    }
}
