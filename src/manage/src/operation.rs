#[macro_export]
macro_rules!  log{
    ($($x:expr),*) =>  {
        || async  {
            let mut data:Vec<String> = Vec::new();
            $(
                data.push(format!("{:?}",$x));
            )*
            let res =  rlp::encode_list::<String,String>(&data);
            let log_canister:ic_cdk::export::Principal = "rrkah-fqaaa-aaaaa-aaaaq-cai".parse().unwrap();
            let sender = ic_cdk::api::caller();
            let res:ic_cdk::api::call::CallResult<()> =  ic_cdk::api::call::call(log_canister,"create_log",(&sender,&res.to_vec(),)).await;
            }
    };
}
