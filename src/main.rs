extern crate chrono;

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::SystemTime;

use chrome_native_messaging::event_loop;
use chrono::DateTime;
use chrono::offset::Utc;
use libtor::{Tor, TorFlag};
use serde::Serialize;
use serde_json::Value as SerdeValue;

#[derive(Serialize)]
struct ResMessage {
    payload: String
}

fn handler(v: SerdeValue) -> Result<ResMessage, String> {
    write_debug(format!("Incoming message: {:?}", &v));
    let response = match get_response(v) {
        Ok(res) => res,
        Err(err) => format!("Error: {:?}", err)
    };
    write_debug(format!("Outgoing message: {:?}", &response));
    Ok(ResMessage { payload: response })
}

fn main() {
    println!("🚧 Debug log: /tmp/alby-rs.log");
    write_debug(format!("Starting Tor"));
    Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::SocksPort(19050))
        .start_background();

    write_debug(format!("Waiting for messages"));
    event_loop(handler);
}

fn get_response(message: SerdeValue) -> Result<String, reqwest::Error> {
    let proxy = reqwest::Proxy::all("socks5h://127.0.0.1:19050")?;
    let client = reqwest::blocking::Client::builder().proxy(proxy).build()?;

    // This code is needed just to "use" a `value` variable
    // we should analyze the message here and define the URL and params, by the message data
    let url = match message.is_boolean() {
        false => "https://facebookwkhpilnemxj7asaniu7vnjjbiltxjqhye3mhbshg7kx5tfyd.onion",
        true => "https://wqskhzt3oiz76dgqbqh27j3qw5aeaui3jxyexzuwxqa5czzo24i3z3ad.onion:8080"
    };
    let res = client.get(url).send()?;
    let body = res.text()?;
    write_debug(format!("onion server response {:?}", &body));
    Ok(body)
}

fn write_debug(msg: String) {
    println!("🐝 {}", &msg);

    let mut file = match OpenOptions::new().append(true).open("/tmp/alby-rs.log") {
        Ok(f) => f,
        Err(_) => match OpenOptions::new().create(true).append(true).open("/tmp/alby-rs.log") {
            Ok(f) => f,
            Err(e) => {
                eprintln!("can't create a log file: {:?}", e);
                return;
            }
        }
    };
    let system_time = SystemTime::now();
    let utc: DateTime<Utc> = system_time.into();
    if let Err(e) = writeln!(file, "{}\t {}", utc.format("%d-%m-%Y %H:%M:%S"), msg) {
        eprintln!("Couldn't write to file: {}", e);
    }
}
