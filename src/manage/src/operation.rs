#[macro_export]
macro_rules! log{
    ($($x:expr),*) => {
        {
            let mut data:Vec<String> = Vec::new();
            $(
                data.push(format!("{:?}",$x));
            )*
            let res =  rlp::encode_list::<String,String>(&data);
            let res =  rlp::decode_list::<String>(&res.to_vec());
           ic_cdk::print(format!("{:?}",res));
        }
    };
}
