use chrome_native_messaging::event_loop;
use serde::Serialize;
use serde_json::Value as SerdeValue;

#[derive(Serialize)]
struct BasicMessage<'a> {
    payload: &'a str
}

fn handler<'a>(_: SerdeValue) -> Result<BasicMessage<'a>, String> {
    Ok(BasicMessage { payload: "Hello, World!" })
}

fn main() {
    event_loop(handler);
}
