use ic_cdk::api::call::call;
use ic_cdk::export::candid::Nat;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;

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
}
#[derive(CandidType, Debug, Deserialize, Clone)]
pub struct CanisterSettings {
    controllers: Option<Vec<Principal>>,
    compute_allocation: Option<Nat>,
    memory_allocation: Option<Nat>,
    freezing_threshold: Option<Nat>,
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
