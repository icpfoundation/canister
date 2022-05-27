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
static mut MANAGE_CANISTER: Principal = Principal::from_slice(&[0]);
static mut OWNER: Principal = Principal::from_slice(&[0]);
#[derive(Hash, PartialEq, Eq, Clone, CandidType, Debug, Deserialize, Copy)]
struct Group {
    user: Principal,
    group_id: u64,
}
type Group_Image_Storage = HashMap<Principal, HashMap<u64, Image>>;
type Project_Image_Storage = HashMap<Group, HashMap<u64, Image>>;
// image max size 200KB
static mut IMAGE_MAX_SIZE: u64 = 204800;
thread_local! {
    static GROUP_IMAGE_STORAGE: RefCell<Group_Image_Storage> = RefCell::default();
    static PROJECT_IMAGE_STORAGE: RefCell<Project_Image_Storage> = RefCell::default();
}

#[derive(CandidType, Debug, Deserialize, Clone)]
struct Image {
    pub size: u64,
    pub ptr: u64,
    pub max_size: u64,
}

#[derive(CandidType, Debug, Deserialize)]
pub struct Member {
    pub name: String,
    pub authority: Authority,
    pub identity: Principal,
    pub join_time: u64,
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

#[init]
fn init(manage_canister: Principal) {
    unsafe {
        OWNER = ic_cdk::caller();
        MANAGE_CANISTER = manage_canister;
    }
}

async fn authority_check(user: Principal, group_id: u64, sender: Principal) -> Result<(), String> {
    unsafe {
        let res: CallResult<(GetGroupMemberInfoRes,)> = call(
            MANAGE_CANISTER,
            "get_group_member_info",
            (user, group_id, sender),
        )
        .await;

        if let GetGroupMemberInfoRes::Ok(member) = res.unwrap().0 {
            return match member.authority {
                Authority::Write | Authority::Operational => return Ok(()),
                _ => return Err("insufficient permissions".to_string()),
            };
        }

        return Err("failed to call get_group_member_info".to_string());
    }
}
#[update]
pub fn update_manage_canister(mange_canister: Principal) {
    let caller = ic_cdk::api::caller();
    unsafe {
        if OWNER != caller {
            ic_cdk::trap("invalid identity");
        }
        MANAGE_CANISTER = mange_canister;
    }
}

#[update]
async fn project_image_store(
    user: Principal,
    group_id: u64,
    project_id: u64,
    data: Vec<u8>,
) -> Result<(), String> {
    let caller = ic_cdk::api::caller();

    let group = Group {
        user: user,
        group_id: group_id,
    };
    unsafe {
        authority_check(user, group_id, caller).await?;

        PROJECT_IMAGE_STORAGE.with(|image_store| {
            let size = data.len() as u64;
            image_store
                .borrow_mut()
                .entry(group)
                .or_insert(HashMap::new());
            if let Some(image_info) = image_store
                .borrow_mut()
                .get_mut(&group)
                .unwrap()
                .get_mut(&project_id)
            {
                if image_info.max_size < size {
                    return Err("the picture is too big".to_string());
                }
                stable64_write(image_info.ptr, &data);
                image_info.size = size;
                return Ok(());
            }

            let image = write_stable(&data, size);
            image_store
                .borrow_mut()
                .get_mut(&group)
                .unwrap()
                .insert(project_id, image);

            Ok(())
        })
    }
}
#[update]
async fn group_image_store(user: Principal, group_id: u64, data: Vec<u8>) -> Result<(), String> {
    let caller = ic_cdk::api::caller();

    unsafe {
        authority_check(user, group_id, caller).await?;
        GROUP_IMAGE_STORAGE.with(|image_store| {
            let size = data.len() as u64;
            image_store
                .borrow_mut()
                .entry(user)
                .or_insert(HashMap::new());
            if let Some(image_info) = image_store
                .borrow_mut()
                .get_mut(&user)
                .unwrap()
                .get_mut(&group_id)
            {
                if image_info.max_size < size {
                    return Err("the picture is too big".to_string());
                }
                stable64_write(image_info.ptr, &data);
                image_info.size = size;
                return Ok(());
            }
            let image = write_stable(&data, size);
            image_store
                .borrow_mut()
                .get_mut(&user)
                .unwrap()
                .insert(group_id, image);

            Ok(())
        })
    }
}

fn write_stable(data: &[u8], size: u64) -> Image {
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
            max_size: size,
        };
        stable64_write(Ptr, &data);
        Ptr = Ptr + size;
        image
    }
}

#[query]
fn get_group_image(user: Principal, group_id: u64) -> Vec<u8> {
    GROUP_IMAGE_STORAGE.with(|image_store| {
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

#[query]
fn get_project_image(user: Principal, group_id: u64, project_id: u64) -> Vec<u8> {
    let group = Group {
        user: user,
        group_id: group_id,
    };
    PROJECT_IMAGE_STORAGE.with(|image_store| {
        let image = image_store
            .borrow()
            .get(&group)
            .unwrap()
            .get(&project_id)
            .unwrap()
            .clone();
        let mut data: Vec<u8> = vec![0; image.size as usize];
        stable64_read(image.ptr, &mut data);
        data
    })
}
