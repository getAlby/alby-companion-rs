extern crate chrono;

use std::{fs, thread};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;

use chrome_native_messaging::event_loop;
use chrono::DateTime;
use libtor::{LogDestination, LogLevel, Tor, TorFlag};
use rand::{Rng, thread_rng};
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
    static TOR_USERNAME: String = format!("u{}", get_random_string());
    static TOR_PASSWORD: String = get_random_string();
    static LOG_FILE: RefCell<String> = RefCell::new(String::from("/tmp/alby-rs.log"));
    static TOR_DIR: RefCell<String> = RefCell::new(String::from("/tmp/tor-rust"));
);

fn main() {
    let args = std::env::args();
    for arg in args {
        if arg.starts_with("--log_file=") || arg.starts_with("--log-file=") || arg.starts_with("-l=") {
            let parts: Vec<&str> = arg.split('=').collect();
            if let Some(val) = parts.get(1) {
                LOG_FILE.with(|v| { *v.borrow_mut() = val.to_string() });
            }
        }
        if arg.starts_with("--tor_dir=") || arg.starts_with("--tor-dir=") || arg.starts_with("-t=") {
            let parts: Vec<&str> = arg.split('=').collect();
            if let Some(val) = parts.get(1) {
                TOR_DIR.with(|v| { *v.borrow_mut() = val.to_string() });
            }
        }
    }

    if !create_lock_file() {
        eprintln!("Only one instance allowed!");
        std::process::exit(1);
    }

    prepare_log_file();
    listen_for_sigterm();
    launch_tor();
    write_debug("Waiting for messages".to_string());
    event_loop(handler);
    remove_lock_file(get_lock_file_path());
}

pub fn launch_tor() {
    let port = get_tor_port();
    let username = get_tor_username();
    let password = get_tor_password();
    let log_file = get_logfile_path();
    let tor_dir = get_tor_dir_path();
    let lock_file = get_lock_file_path();
    write_debug(format!("Starting Tor on port {}, user: {}, in folder {}. Log redirected to {}", port, username, &tor_dir, &log_file));

    thread::spawn(move || {
        let tor_thread = Tor::new()
            .flag(TorFlag::DataDirectory(tor_dir))
            .flag(TorFlag::ControlPort(0))
            .flag(TorFlag::LogTo(LogLevel::Notice, LogDestination::File(log_file.clone())))
            .flag(TorFlag::Quiet())
            .flag(TorFlag::Socks5ProxyUsername(username))
            .flag(TorFlag::Socks5ProxyPassword(password))
            .flag(TorFlag::SocksPort(port))
            .start_background();
        if wait_for_tor(20, &log_file) {
            send_tor_started_msg();
        }
        match tor_thread.join() {
            Ok(r) => match r {
                Ok(result) => {
                    write_debug_to(format!("Tor thread was terminated: {}", result), &log_file);
                    send_stdout_msg(ResMessage {
                        id: "status".to_string(),
                        status: result as u16,
                        body: "terminate".to_string(),
                        headers: HashMap::from([
                            (String::from("X-Alby-internal"), String::from("true")),
                            (String::from("X-Alby-description"), String::from("Tor thread was terminated"))
                        ])
                    });
                    exit(result as i32, lock_file);
                },
                Err(err) => {
                    write_debug_to(format!("Can not spawn Tor thread: {:?}", err), &log_file);
                    send_stdout_msg(ResMessage {
                        id: "status".to_string(),
                        status: 502,
                        body: "error".to_string(),
                        headers: HashMap::from([
                            (String::from("X-Alby-internal"), String::from("true")),
                            (String::from("X-Alby-description"), String::from("Can not spawn Tor thread"))
                        ])
                    });
                    exit(1, lock_file);
                }
            },
            Err(_) => write_debug(String::from("Tor thread has panicked"))
        }
    });
}

fn send_tor_started_msg() {
    send_stdout_msg(ResMessage {
        id: "status".to_string(),
        status: 100,
        body: "tor_started".to_string(),
        headers: HashMap::from([("X-Alby-Internal".to_string(), "true".to_string())])
    });
}

fn send_stdout_msg(msg: ResMessage) -> bool {
    chrome_native_messaging::send_message(std::io::stdout(), &msg).is_ok()
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

pub fn prepare_log_file() {
    let path = get_logfile_path();
    if Path::new(&path).exists() {
        match fs::remove_file(&path) {
            Ok(_) => (),
            Err(e) => eprintln!("can't prepare a log file {}: {:?}", path, e)
        }
    }
}

fn write_debug(msg: String) {
    write_debug_to(msg, &get_logfile_path());
}

fn write_debug_to(msg: String, log_file: &str) {
    let mut file = match OpenOptions::new().append(true).open(log_file) {
        Ok(f) => f,
        Err(_) => match OpenOptions::new().create(true).append(true).open(log_file) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("can't create a log file {}: {:?}", log_file, e);
                return;
            }
        }
    };
    if let Err(e) = writeln!(file, "{}\t {}", get_system_time(), msg) {
        eprintln!("Couldn't write to log file: {}", e);
    }
}

fn get_random_port() -> u16 {
    let mut rng = thread_rng();
    rng.gen_range(19050..29051)
}

fn get_random_string() -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(10)
        .collect()
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
        (*(v.borrow())).clone()
    })
}

fn get_tor_dir_path() -> String {
    TOR_DIR.with(|v| {
        (*(v.borrow())).clone()
    })
}

fn get_lock_file_path() -> String {
    format!("{}.process", get_logfile_path())
}

fn listen_for_sigterm() {
    let lock_file = get_lock_file_path();
    match Signals::new(TERM_SIGNALS) {
        Ok(mut signals) => {
            thread::spawn(move || {
                for _ in signals.forever() {
                    write_debug("SIGTERM received".to_string());
                    exit(0, lock_file.clone());
                }
            });
        },
        Err(e) => write_debug(format!("Can not start signals listener: {:?}", e))
    }
}

fn create_lock_file() -> bool {
    match OpenOptions::new().write(true)
        .create_new(true)
        .open(get_lock_file_path()) {
        Ok(mut file) => {
            let _ = writeln!(file, "{}\t {}", get_system_time(), std::process::id());
            true
        },
        Err(err) => {
            eprintln!("Can not create a lock file: {:?}", err);
            false
        }
    }
}

fn get_system_time() -> String {
    let system_time = SystemTime::now();
    let dt: DateTime<chrono::Local> = system_time.into();
    dt.format("%d-%m-%Y %H:%M:%S").to_string()
}

fn remove_lock_file(path: String) {
    let _ = fs::remove_file(path);
}

fn exit(code: i32, lock_file: String) {
    remove_lock_file(lock_file);
    std::process::exit(code);
}

pub fn wait_for_tor(seconds: u8, log_file: &str) -> bool {
    for _ in 0..seconds {
        let log = get_log(log_file);
        if log.contains("Bootstrapped 100% (done):") {
            return true;
        }
        thread::sleep(std::time::Duration::from_secs(1));
    }
    false
}

pub fn get_log(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => String::new()
    }
}