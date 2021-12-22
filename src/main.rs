extern crate chrono;

// use std::{thread, time};
use std::borrow::Borrow;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::SystemTime;

use chrome_native_messaging::event_loop;
use chrono::DateTime;
use chrono::offset::Utc;
use libtor::{Tor, TorFlag};
use rand::{random, Rng, thread_rng};
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

thread_local!(
    static TOR_PORT: u16 = get_random_port();
    static TOR_USERNAME: String = get_random_string();
    static TOR_PASSWORD: String = get_random_string();
);

fn main() {
    println!("🚧 Debug log: /tmp/alby-rs.log");
    write_debug(format!("Starting Tor on {}", get_tor_port()));

    Tor::new()
        .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
        .flag(TorFlag::ControlPort(0))
        .flag(TorFlag::Socks5ProxyUsername(get_tor_username()))
        .flag(TorFlag::Socks5ProxyPassword(get_tor_password()))
        .flag(TorFlag::SocksPort(get_tor_port()))
        .start_background();

    write_debug("Waiting for messages".to_string());

    // thread::sleep(time::Duration::from_secs(10));
    // match get_response(SerdeValue::from("")) {
    //     Ok(_) => (),
    //     Err(e) => eprintln!("e: {:?}", e)
    // }
    event_loop(handler);
}

fn get_response(message: SerdeValue) -> Result<String, reqwest::Error> {
    let proxy = reqwest::Proxy::all(&format!("socks5h://127.0.0.1:{}", get_tor_port()))?
        .basic_auth(&get_tor_username(), &get_tor_password());
    let client = reqwest::blocking::Client::builder().proxy(proxy).build()?;

    // This code is needed just to "use" a `value` variable
    // we should analyze the message here and define the URL and params, by the message data
    let url = match message.is_boolean() {
        false => "https://facebookwkhpilnemxj7asaniu7vnjjbiltxjqhye3mhbshg7kx5tfyd.onion",
        true => "https://wqskhzt3oiz76dgqbqh27j3qw5aeaui3jxyexzuwxqa5czzo24i3z3ad.onion:8080"
    };
    let res = client.get(url).send()?;
    let status = res.status();
    let body = res.text()?;
    write_debug(format!("onion server response status: {}, length: {}", status, &body.len()));
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

fn get_random_port() -> u16 {
    let mut rng = thread_rng();
    rng.gen_range(19050..29051)
}

fn get_random_string() -> String {
    (0..10).map(|_| random::<char>()).collect()
}

fn get_tor_port() -> u16 {
    TOR_PORT.with(|tor_port| *tor_port.borrow())
}

fn get_tor_username() -> String {
    TOR_USERNAME.with(|v| {
        let s: &str = v.borrow();
        s.to_string()
    })
}

fn get_tor_password() -> String {
    TOR_PASSWORD.with(|v| {
        let s: &str = v.borrow();
        s.to_string()
    })
}
