use ic_cdk_macros::*;

#[query]
pub fn hello_world() {
    ic_cdk::print("hello world");
}
