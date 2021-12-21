use chrome_native_messaging::event_loop;
use libtor::{Tor, TorFlag};
use serde::Serialize;
use serde_json::Value as SerdeValue;

#[derive(Serialize)]
struct BasicMessage {
    payload: String
}

fn handler(v: SerdeValue) -> Result<BasicMessage, String> {
    match get_response(v) {
        Ok(response) => Ok(BasicMessage { payload: response }),
        Err(err) => Ok(BasicMessage { payload: format!("Error: {:?}", err) })
    }
}

fn main() {
    println!("Starting Tor");
    Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::SocksPort(19050))
        .start_background();

    println!("Waiting for messages");
    event_loop(handler);
}

fn get_response(value: SerdeValue) -> Result<String, reqwest::Error> {
    let proxy = reqwest::Proxy::all("socks5h://127.0.0.1:19050")?;
    let client = reqwest::blocking::Client::builder().proxy(proxy).build()?;

    // This code is needed just to "use" a `value` variable
    let url = match value.is_boolean() {
        false => "https://facebookwkhpilnemxj7asaniu7vnjjbiltxjqhye3mhbshg7kx5tfyd.onion",
        true => "https://wqskhzt3oiz76dgqbqh27j3qw5aeaui3jxyexzuwxqa5czzo24i3z3ad.onion:8080"
    };
    let res = client.get(url).send()?;
    let body = res.text()?;
    println!("onion server response {:?}", &body);
    Ok(body)
}
