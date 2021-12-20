use std::io;

use chrome_native_messaging::event_loop;
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct BasicMessage<'a> {
    payload: &'a str
}

fn main() {
    event_loop(|value| match value {
        Null => Err("null payload"),
        _ => Ok(BasicMessage { payload: "Hello, World!" })
    });
}
