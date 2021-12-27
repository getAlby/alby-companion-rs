extern crate chrono;

use std::{fs, thread};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;

use chrome_native_messaging::event_loop;
use chrono::DateTime;
use rand::{Rng, thread_rng};
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::iterator::Signals;

#[cfg(test)]
mod test;

mod messages;
mod tor;
mod requests;

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
    crate::tor::launch_tor();
    write_debug("Waiting for messages".to_string());
    event_loop(crate::messages::handler);
    remove_lock_file(get_lock_file_path());
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


pub fn get_log(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => String::new()
    }
}