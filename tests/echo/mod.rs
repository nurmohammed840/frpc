use frpc::declare;
use frpc_macros::{Input, Message};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    sync::atomic::{AtomicBool, Ordering},
};

fn println(value: &dyn Debug) {
    println!("{:#?}", value);
}

macro_rules! def {
    [$($id: literal fn $name:ident -> $ty: ty)*] => {
        $(
            async fn $name(state: State, value: $ty) -> $ty {
                if state.log_lvl.load(Ordering::Acquire) { println(&value); }
                value
            }
        )*
        declare! {
            pub service EchoTest {
                type State = Arc<Context>;
                rpc log = 0;
                $(rpc $name = $id;)*
            }
        }
    };
}

#[derive(Debug, Default)]
pub struct Context {
    log_lvl: AtomicBool,
}

type State = frpc::State<Arc<Context>>;

#[derive(Input, Debug)]
enum Log {
    Disable,
    Enable,
}

async fn log(state: State, log: Log) {
    match log {
        Log::Enable => state.log_lvl.store(true, Ordering::Relaxed),
        Log::Disable => state.log_lvl.store(false, Ordering::Relaxed),
    }
}

#[derive(Debug, Message)]
struct Bufs {
    vec_2d: [f32; 2],
    vec_3d: [f64; 3],

    floats: Vec<f32>,
    long_floats: Vec<f64>,

    big_nums: Vec<i64>,
    sorted_nums: BTreeSet<i8>,
    bytes: Vec<u8>,
}

def! {
    // Number
    1 fn echo_u8 -> u8
    2 fn echo_u16 -> u16
    3 fn echo_u32 -> u32
    4 fn echo_u64 -> u64
    5 fn echo_u128 -> u128
    6 fn echo_usize -> usize

    // Neg Number
    7 fn echo_i8 -> i8
    8 fn echo_i16 -> i16
    9 fn echo_i32 -> i32
    10 fn echo_i64 -> i64
    11 fn echo_i128 -> i128
    12 fn echo_isize -> isize

    // Flote Number
    13 fn echo_f32 -> f32
    14 fn echo_f64 -> f64

    // other
    15 fn echo_option -> Option<Option<&str>>
    16 fn echo_result -> Result<String, String>

    // -----------------

    17 fn echo_str -> &str
    18 fn echo_bool -> bool

    // -----------------

    19 fn echo_map -> HashMap<&str, f64>
    20 fn echo_sorted_map -> BTreeMap<&str, f64>

    21 fn echo_bufs -> Bufs
}
