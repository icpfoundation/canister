use ic_cdk::export::Principal;
#[macro_export]
macro_rules!  log{
    ($user:expr,$group_id:expr,$sender:expr,$action:expr, $($x:expr),*) =>  {
        || async  {
            let mut data:Vec<String> = Vec::new();
            $(
                data.push(format!("{:?}",$x));
            )*
            let res =  rlp::encode_list::<String,String>(&data);
            unsafe{
                let user = Principal::from_text($user).unwrap();
                let sender = Principal::from_text($sender).unwrap();
                let group_id:u64 = $group_id.try_into().unwrap();
                let res:ic_cdk::api::call::CallResult<()> =  ic_cdk::api::call::call(crate::constant::LOG_CANISTER,"create_log",(&user,&group_id,&sender,$action,&res.to_vec(),)).await;
            }
            }
    };
}
