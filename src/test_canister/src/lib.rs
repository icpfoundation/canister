use ic_cdk_macros::*;
ic_cdk::export::candid::export_service!();
#[query]
pub fn hello_world() {
    ic_cdk::print("hello world");
}

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    include_str!("./test_canister.did").to_string()
}
