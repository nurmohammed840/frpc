#![allow(non_camel_case_types)]

use frpc_macros::Message;

type DataType = (((), ((), ())), r#class, r#enum);

async fn r#get_data() -> DataType {
    (
        // Empty typles
        ((), ((), ())),
        r#class { r#new: () },
        r#enum::r#type,
    )
}

async fn validate(_data: DataType) {
    assert!(_data == get_data().await);
}

#[derive(Message, PartialEq)]
struct r#class {
    r#new: (),
}

#[derive(Message, PartialEq)]
enum r#enum {
    r#type,
}

frpc::declare! {
    pub service r#ValidateTest {
        rpc r#get_data = 1;
        rpc validate = 2;
    }
}
