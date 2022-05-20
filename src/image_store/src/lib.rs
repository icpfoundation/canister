use ic_cdk::api::call::call;
use ic_cdk::api::call::CallResult;
use ic_cdk::api::stable::{stable64_grow, stable64_read, stable64_size, stable64_write};
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;
static mut Ptr: u64 = 0;
static mut Page: u64 = 0;
const PAGE_SIZE: u64 = 65536;
static mut MANAGE_CANISTER: &str = "r7inp-6aaaa-aaaaa-aaabq-cai";
type Image_Storage = HashMap<Principal, HashMap<u64, Image>>;
thread_local! {
    static IMAGE_STORAGE: RefCell<Image_Storage> = RefCell::default();
}

#[derive(CandidType, Debug, Deserialize, Clone)]
struct Image {
    pub size: u64,
    pub ptr: u64,
}

#[derive(CandidType, Debug, Deserialize)]
pub struct Member {
    pub name: String,
    pub authority: Authority,
    pub identity: Principal,
}
// Project permission management is similar to Linux file operation permission
#[derive(CandidType, Debug, Deserialize)]
pub enum Authority {
    // You can read the basic information of groups or projects, but you cannot modify them
    Read,
    // You can read and modify the basic information of the project,
    // but you can't operate more core functions, such as deleting canisters
    Write,
    // You can do anything
    Operational,
}
#[derive(CandidType, Debug, Deserialize)]
enum GetGroupMemberInfoRes {
    Ok(Member),
    Err(String),
}

#[update]
async fn image_store(canister: Principal, user: Principal, group_id: u64, data: Vec<u8>) {
    let caller = ic_cdk::api::caller();
    unsafe {
        let manage_canister = Principal::from_text(MANAGE_CANISTER).unwrap();
        let res: CallResult<(GetGroupMemberInfoRes,)> =
            call(canister, "get_group_member_info", (user, group_id, caller)).await;
        if let GetGroupMemberInfoRes::Ok(member) = res.unwrap().0 {
            if let Authority::Write = member.authority {
                IMAGE_STORAGE.with(|image_store| {
                    image_store
                        .borrow_mut()
                        .entry(user)
                        .or_insert(HashMap::new());
                    if image_store
                        .borrow()
                        .get(&user)
                        .unwrap()
                        .contains_key(&group_id)
                    {
                        ic_cdk::trap("pictures have been stored");
                    }

                    let size = data.len() as u64;
                    unsafe {
                        if Ptr + size > Page * PAGE_SIZE {
                            let page = (Ptr + size) / PAGE_SIZE + 1;
                            if page > Page {
                                let grow = page - Page;
                                stable64_grow(grow);
                                Page = page;
                            }
                        }
                        let image = Image {
                            ptr: Ptr,
                            size: size,
                        };
                        stable64_write(Ptr, &data);
                        Ptr = Ptr + size;
                        image_store
                            .borrow_mut()
                            .get_mut(&user)
                            .unwrap()
                            .insert(group_id, image);
                    }
                });
            }
        }
    }
}

#[query]
fn get_image(user: Principal, group_id: u64) -> Vec<u8> {
    IMAGE_STORAGE.with(|image_store| {
        let image = image_store
            .borrow()
            .get(&user)
            .unwrap()
            .get(&group_id)
            .unwrap()
            .clone();
        let mut data: Vec<u8> = vec![0; image.size as usize];
        stable64_read(image.ptr, &mut data);
        data
    })
}
