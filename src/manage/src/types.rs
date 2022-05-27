use ic_cdk::export::candid::{CandidType, Deserialize};
#[derive(CandidType, Debug, Deserialize, Clone, PartialEq, Eq)]
pub enum Profile {
    Public,
    Private,
}

#[derive(CandidType, Debug, Deserialize, Clone)]
pub enum Action {
    UpdateGroup(u64, String),
    UpdateProject(u64, u64, String),
    UpdateProjectCanister(u64, u64, String),
}
#[derive(CandidType, Debug, Deserialize, Clone)]
pub enum ProjectType {
    Wallet,
    Infrastructure,
    NFT,
    Authentication,
    Tools,
    Dapp,
    Sns,
    Defi,
    Games,
    Official,
    Bridge,
    Swap,
    Metaverse,
}
