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

#[derive(CandidType, Debug, Deserialize, Clone)]
pub enum InstallCodeMode {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "reinstall")]
    Reinstall,
    #[serde(rename = "upgrade")]
    Upgrade,
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
            Err((code, msg)) => {
                return Err(format!(
                    "get_canister_status faile: {}: {}",
                    code as u8, msg
                ));
            }
        }
    }

    pub async fn stop_canister(canister: Principal) -> Result<(), String> {
        let canister_id = CanisterIdRecord {
            canister_id: canister,
        };

        match call(
            Principal::management_canister(),
            "stop_canister",
            (canister_id,),
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err((code, msg)) => {
                return Err(format!(
                    "get_canister_status faile: {}: {}",
                    code as u8, msg
                ));
            }
        }
    }

    pub async fn start_canister(canister: Principal) -> Result<(), String> {
        let canister_id = CanisterIdRecord {
            canister_id: canister,
        };

        match call(
            Principal::management_canister(),
            "start_canister",
            (canister_id,),
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err((code, msg)) => {
                return Err(format!(
                    "get_canister_status faile: {}: {}",
                    code as u8, msg
                ));
            }
        }
    }

    pub async fn delete_canister(canister: Principal) -> Result<(), String> {
        let canister_id = CanisterIdRecord {
            canister_id: canister,
        };

        match call(
            Principal::management_canister(),
            "delete_canister",
            (canister_id,),
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err((code, msg)) => {
                return Err(format!(
                    "get_canister_status faile: {}: {}",
                    code as u8, msg
                ));
            }
        }
    }

    pub async fn install_code(
        canister: Principal,
        install_mod: InstallCodeMode,
        wasm: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<(), String> {
        let canister_id = CanisterIdRecord {
            canister_id: canister,
        };

        match call(
            Principal::management_canister(),
            "install_code",
            (install_mod, canister_id, wasm, args),
        )
        .await
        {
            Ok(()) => return Ok(()),
            Err((code, msg)) => {
                return Err(format!(
                    "get_canister_status faile: {}: {}",
                    code as u8, msg
                ));
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
