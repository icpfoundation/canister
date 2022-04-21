#[macro_use]
macro_rules! emit{
    ($($x:expr),*) => {
        {
            let mut data = Vec::new();
            $(
                data.push(format!("{:?}",$x));
            )*
           let res =  rlp::encode_list(data);
        }

    };
}
