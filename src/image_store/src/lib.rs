use ic_cdk::api::stable::{stable64_grow, stable64_read, stable64_size, stable64_write};
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_cdk_macros::*;
use std::cell::RefCell;
use std::collections::HashMap;
#[derive(CandidType, Debug, Deserialize, Clone)]
struct Image {
    pub size: u64,
    pub ptr: u64,
}
type Image_Storage = HashMap<String, Image>;
use ic_cdk::api::call::call;
static mut Ptr: u64 = 0;
static mut Page: u64 = 0;
const PAGE_SIZE: u64 = 65536;
thread_local! {
    static IMAGE_STORAGE: RefCell<Image_Storage> = RefCell::default();
}

#[update]
async fn image_store(canister: Principal, cid: String, data: Vec<u8>) {
    IMAGE_STORAGE.with(|image_store| {
        if image_store.borrow().contains_key(&cid) {
            ic_cdk::trap("cid already exists");
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
            image_store.borrow_mut().insert(cid, image);
        }
    });
}

#[query]
fn get_image(cid: String) -> Vec<u8> {
    IMAGE_STORAGE.with(|image_store| {
        let image = image_store.borrow().get(&cid).unwrap().clone();
        let mut data: Vec<u8> = vec![0; image.size as usize];
        stable64_read(image.ptr, &mut data);
        data
    })
}
