use frpc::*;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct User {
    name: Mutex<Option<String>>,
}

async fn whats_my_name(user: State<Arc<User>>) -> String {
    match &*user.name.lock().unwrap() {
        Some(what) => format!("Your name is {what}!"),
        None => "I don't know. I am a tee pot!".into(),
    }
}

fn my_name_is(user: State<Arc<User>>, what: String) -> impl Output {
    user.name.lock().unwrap().replace(what.clone());
    Return(what + " you are!")
}

declare! {
    pub service Stateful {
        type State = Arc<User>;

        rpc my_name_is = 1;
        rpc whats_my_name = 2;
    }
}
