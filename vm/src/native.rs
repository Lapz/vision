use std::time::{SystemTime, UNIX_EPOCH};

use crate::Value;

pub fn clock_native(_arg_count: usize, _args: *const Value) -> Value {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    Value::number(time.as_secs() as f64 + f64::from(time.subsec_nanos()) * 1e-9)
}
