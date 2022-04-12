use ic_cdk::api::call::call;
use ic_cdk::export::candid::Nat;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct CanisterIdRecord {
    pub canister_id: Principal,
}

#[derive(CandidType, Debug, Deserialize, Clone)]
pub enum CanisterStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "stopped")]
    Stopped,
}

#[derive(CandidType, Debug, Deserialize)]
pub struct CanisterStatusResponse {
    pub status: CanisterStatus,
    pub settings: CanisterSettings,
    pub module_hash: Option<Vec<u8>>,
    pub memory_size: Nat,
    pub cycles: Nat,
}

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct ManageCanister {
    canister_id: Principal,
    settings: CanisterSettings,
}
impl ManageCanister {
    pub fn new(canister_id: Principal, settings: CanisterSettings) -> Self {
        Self {
            canister_id: canister_id,
            settings: settings,
        }
    }
    pub async fn set_controller(self) -> Result<(), String> {
        match call(Principal::management_canister(), "update_settings", (self,)).await {
            Ok(()) => {
                return Ok(());
            }
            Err((code, msg)) => {
                return Err(format!("update canister faile: {}: {}", code as u8, msg));
            }
        }
    }

    pub async fn get_canister_status(
        canister: Principal,
    ) -> Result<CanisterStatusResponse, String> {
        let canister_id = CanisterIdRecord {
            canister_id: canister,
        };

        match call(
            Principal::management_canister(),
            "canister_status",
            (canister_id,),
        )
        .await
        {
            Ok((status,)) => return Ok(status),
            Err(err) => {
                let err = format!("{:?}", err);
                return Err(err);
            }
        }
    }
}

#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct CanisterSettings {
    pub controllers: Option<Vec<Principal>>,
    pub compute_allocation: Option<Nat>,
    pub memory_allocation: Option<Nat>,
    pub freezing_threshold: Option<Nat>,
}

impl CanisterSettings {
    pub fn new(
        controllers: Option<Vec<Principal>>,
        compute_allocation: Option<Nat>,
        memory_allocation: Option<Nat>,
        freezing_threshold: Option<Nat>,
    ) -> Self {
        Self {
            controllers: controllers,
            compute_allocation: compute_allocation,
            memory_allocation: memory_allocation,
            freezing_threshold: freezing_threshold,
        }
    }
}
