extern crate chrono;

use std::{fs, thread};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::SystemTime;

use chrome_native_messaging::event_loop;
use chrono::DateTime;
use chrono::offset::Utc;
use libtor::{LogDestination, LogLevel, Tor, TorFlag};
use rand::{random, Rng, thread_rng};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as SerdeValue;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::iterator::Signals;

#[cfg(test)]
mod test;

#[derive(Deserialize, Debug)]
struct ReqMessage {
    id: String,
    url: String,
    method: String,
    body: Option<String>,
    params: Option<HashMap<String, String>>,
    headers: Option<HashMap<String, String>>,
}

#[derive(Serialize, Debug)]
struct ResMessage {
    id: String,
    status: u16,
    body: String,
    headers: HashMap<String, String>,
}

impl Default for ReqMessage {
    fn default() -> ReqMessage {
        ReqMessage {
            id: String::new(),
            url: String::new(),
            method: String::from("GET"),
            body: None,
            params: None,
            headers: None,
        }
    }
}

fn handler(v: SerdeValue) -> Result<ResMessage, String> {
    write_debug(format!("Incoming message: {:?}", &v));
    let msg: ReqMessage = match serde_json::from_value::<ReqMessage>(v) {
        Ok(m) => m,
        Err(err) => return Err(format!("Can not parse message: {:?}", err))
    };
    let response = match get_response(msg) {
        Ok(res) => res,
        Err(err) => return Err(format!("Can not get response from resource: {:?}", err))
    };
    write_debug(format!("Outgoing message: {:?}", &response));
    Ok(response)
}

thread_local!(
    static TOR_PORT: u16 = get_random_port();
    static TOR_USERNAME: String = get_random_string();
    static TOR_PASSWORD: String = get_random_string();
    static LOG_FILE: String = String::from("/tmp/alby-rs.log")
);

fn main() {
    listen_for_sigterm();
    launch_tor();
    prepare_log_file();
    write_debug("Waiting for messages".to_string());
    event_loop(handler);
}

fn launch_tor() {
    let port = get_tor_port();
    let username = get_tor_username();
    let password = get_tor_password();
    write_debug(format!("Starting Tor on {}", port));

    thread::spawn(move || {
        let tor_thread = Tor::new()
            .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
            .flag(TorFlag::ControlPort(0))
            .flag(TorFlag::LogTo(LogLevel::Notice, LogDestination::File(get_logfile_path())))
            .flag(TorFlag::Quiet())
            .flag(TorFlag::Socks5ProxyUsername(username))
            .flag(TorFlag::Socks5ProxyPassword(password))
            .flag(TorFlag::SocksPort(port))
            .start_background();
        let _ = tor_thread.join().expect("Tor thread has panicked");
        write_debug("Tor thread was terminated".to_string());
    });
}

#[derive(Debug)]
pub enum ReqError {
    ClientError(reqwest::Error),
    Message(String),
}

impl From<reqwest::Error> for ReqError {
    fn from(err: reqwest::Error) -> Self {
        ReqError::ClientError(err)
    }
}

fn get_response(message: ReqMessage) -> Result<ResMessage, ReqError> {
    let proxy = reqwest::Proxy::all(&format!("socks5h://127.0.0.1:{}", get_tor_port()))?
        .basic_auth(&get_tor_username(), &get_tor_password());
    let client = reqwest::blocking::Client::builder().danger_accept_invalid_certs(true).proxy(proxy).build()?;
    let method = match message.method.as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        _ => reqwest::Method::OPTIONS,
    };

    let headers: HeaderMap = match message.headers {
        Some(map) => match HeaderMap::try_from(&map) {
            Ok(h) => h,
            Err(_) => {
                write_debug("headers were not parsed".to_string());
                HeaderMap::new()
            }
        },
        None => HeaderMap::new(),
    };

    let mut url = match reqwest::Url::parse(&message.url) {
        Ok(u) => u,
        Err(err) => return Err(ReqError::Message(format!("Can not parse URL: {}", err)))
    };
    if let Some(params_map) = message.params {
        let mut pairs = url.query_pairs_mut();
        for (param, value) in params_map.into_iter() {
            pairs.append_pair(&param, &value);
        }
    }

    let res = client.request(method, url).headers(headers).body(message.body.unwrap_or_default()).send()?;
    let status = res.status();
    let mut res_headers: HashMap<String, String> = HashMap::new();
    for (header_name, header_value) in res.headers().into_iter() {
        res_headers.insert(header_name.to_string(), header_value.to_str().unwrap_or("[can not be converted into string]").to_string());
    }
    let body = res.text()?;
    write_debug(format!("onion server response status: {}, length: {}", &status, &body.len()));

    Ok(ResMessage {
        id: message.id,
        status: status.into(),
        body,
        headers: res_headers
    })
}

fn prepare_log_file() {
    match fs::remove_file(get_logfile_path()) {
        Ok(_) => (),
        Err(e) => eprintln!("can't prepare a log file {}: {:?}", get_logfile_path(), e)
    }
}

fn write_debug(msg: String) {
    let mut file = match OpenOptions::new().append(true).open(get_logfile_path()) {
        Ok(f) => f,
        Err(_) => match OpenOptions::new().create(true).append(true).open(get_logfile_path()) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("can't create a log file {}: {:?}", get_logfile_path(), e);
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

fn get_logfile_path() -> String {
    LOG_FILE.with(|v| {
        let s: &str = v.borrow();
        s.to_string()
    })
}

fn listen_for_sigterm() {
    match Signals::new(TERM_SIGNALS) {
        Ok(mut signals) => {
            thread::spawn(move || {
                for _ in signals.forever() {
                    write_debug("SIGTERM received".to_string());
                    std::process::exit(0);
                }
            });
        },
        Err(e) => write_debug(format!("Can not start signals listener: {:?}", e))
    }
}