macro_rules! emit {
    ($($x:expr),*) => {
        $(
            println!("{:?}", $x);
        )*

    };
}

#[cfg(test)]
mod emit_test {
    use super::*;
    #[test]
    fn test_emit() {
        #[derive(Debug)]
        pub struct N {
            pub n: String,
        }
        let n = N {
            n: "hjel".to_string(),
        };
        emit!(n);
    }
}
