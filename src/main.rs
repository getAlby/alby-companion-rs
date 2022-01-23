// extern crate chrono;

use std::{fs, thread};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::time::SystemTime;

use chrome_native_messaging::event_loop;
use chrono::DateTime;
use rand::{Rng, thread_rng};
#[cfg(not(windows))]
use signal_hook::consts::TERM_SIGNALS;
#[cfg(not(windows))]
use signal_hook::iterator::Signals;
use sysinfo::{PidExt, System, SystemExt};

#[cfg(test)]
mod test;

mod messages;
mod tor;
mod requests;
mod cli;

thread_local!(
    static TOR_PORT: u16 = get_random_port();
    static TOR_USERNAME: String = format!("u{}", get_random_string());
    static TOR_PASSWORD: String = get_random_string();
    static LOG_FILE: RefCell<String> = RefCell::new(format!("{}{}",std::env::temp_dir().to_string_lossy(),"alby.log"));
    static TOR_DIR: RefCell<String> = RefCell::new(format!("{}{}",std::env::temp_dir().to_string_lossy(),"alby-tor"));
    static TOR_STARTED: RefCell<bool> = RefCell::new(false);
    static TOR_READY: RefCell<bool> = RefCell::new(false);
    static DEBUG_MODE: RefCell<bool> = RefCell::new(false);
);

fn main() {
    let opts = cli::get_cli_options(cli::get_args_from_cli());
    if let Some(val) = opts.log_file {
        LOG_FILE.with(|v| { *v.borrow_mut() = val.to_string() });
    }
    if let Some(val) = opts.tor_dir {
        TOR_DIR.with(|v| { *v.borrow_mut() = val.to_string() });
    }
    if opts.debug_mode {
        set_debug_mode(true);
    }

    let lock = create_lock_file();
    if lock.is_none() {
        eprintln!("Only one instance allowed!");
        std::process::exit(1);
    }

    prepare_log_file();
    listen_for_sigterm();
    write_debug("Waiting for messages");
    event_loop(messages::handler);
}


pub fn prepare_log_file() -> bool {
    let path = get_logfile_path();
    let debug_mode = is_debug_mode();
    if Path::new(&path).exists() {
        match fs::remove_file(&path) {
            Ok(_) => {
                if debug_mode {
                    write_debug_to(format!("Log file prepared: {}", &path), &path, debug_mode);
                }
                write_debug_to(get_pid_key(), &path, debug_mode)
            },
            Err(e) => {
                eprintln!("can't prepare a log file {}: {:#?}", path, e);
                false
            }
        }
    } else if write_debug_to(get_pid_key(), &path, debug_mode) {
        if debug_mode {
            write_debug_to(format!("Log file created: {}", &path), &path, debug_mode);
        }
        true
    } else {
        eprintln!("can't prepare a log file {}!", path);
        false
    }
}

pub fn get_pid_key() -> String {
    format!("process: {}", std::process::id())
}

#[allow(unused_results)]
fn write_debug<T: Display>(msg: T) -> bool {
    write_debug_to(msg, &get_logfile_path(), is_debug_mode())
}

#[allow(unused_results)]
fn write_debug_to<T: Display>(msg: T, log_file: &str, debug_mode: bool) -> bool {
    if debug_mode {
        eprintln!("🚧 {}", &msg);
    }
    let mut file = match OpenOptions::new().append(true).open(log_file) {
        Ok(f) => f,
        Err(_) => match OpenOptions::new().create(true).append(true).open(log_file) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("can't create a log file {}: {:#?}", log_file, e);
                return false;
            }
        }
    };
    if let Err(e) = writeln!(file, "{}\t {}", get_system_time(), msg) {
        eprintln!("Couldn't write to log file: {}", e);
        return false;
    }
    true
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

#[cfg(not(windows))]
fn listen_for_sigterm() {
    let lock_file = get_lock_file_path();
    let log_file = get_logfile_path();
    let debug_mode = is_debug_mode();
    match Signals::new(TERM_SIGNALS) {
        Ok(mut signals) => {
            thread::spawn(move || {
                for _ in signals.forever() {
                    write_debug_to("SIGTERM received", &log_file, debug_mode);
                    exit(0, lock_file.clone());
                }
            });
        },
        Err(e) => {
            write_debug(format!("Can not start signals listener: {:#?}", e));
        },
    }
}

#[cfg(windows)]
fn listen_for_sigterm() {}

struct LockFile {
    path: String,
}

impl Drop for LockFile {
    fn drop(&mut self) {
        remove_lock_file(self.path.to_string());
    }
}

fn create_lock_file() -> Option<LockFile> {
    let path = get_lock_file_path();
    // Log file is not prepared at this moment, 
    // so all debug info should be printed to std_err.
    let debug_mode = is_debug_mode();

    if Path::new(&path).exists() {
        match fs::read_to_string(&path) {
            Ok(pid_str) => match pid_str.trim().parse::<u32>() {
                Err(err) => {
                    if debug_mode {
                        eprintln!("⚠️ Can't parse PID from the lock file [{}]: {:#?}", &path, err);
                    }
                    return None;
                },
                Ok(pid) => match is_pid_exists(pid) {
                    true => return None,
                    false => {
                        if let Err(err) = fs::remove_file(&path) {
                            if debug_mode {
                                eprintln!("⚠️ Can't remove lock file [{}] of non-existing process [{}]: {:#?}", &path, pid, err);
                            }
                        }
                    }
                }
            },
            Err(err) => {
                if debug_mode {
                    eprintln!("⚠️ Can't read PID from the lock file [{}]: {:#?}", &path, err);
                }
                return None;
            }
        }
    }


    match OpenOptions::new().write(true)
        .create_new(true)
        .open(&path) {
        Ok(mut file) => {
            let _ = write!(file, "{}", std::process::id());
            Some(LockFile { path })
        },
        Err(err) => {
            eprintln!("Can not create a lock file: {:#?}", err);
            None
        }
    }
}

fn is_pid_exists(pid: u32) -> bool {
    let sys = System::new_all();
    sys.process(sysinfo::Pid::from_u32(pid)).is_some()
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

pub fn is_tor_started() -> bool {
    TOR_STARTED.with(|v| *v.borrow())
}

pub fn set_tor_is_started(val: bool) {
    TOR_STARTED.with(|v| *v.borrow_mut() = val)
}

pub fn is_tor_ready() -> bool {
    TOR_READY.with(|v| *v.borrow())
}

pub fn set_tor_is_ready(val: bool) {
    TOR_READY.with(|v| *v.borrow_mut() = val)
}

pub fn is_debug_mode() -> bool {
    DEBUG_MODE.with(|v| *v.borrow())
}

pub fn set_debug_mode(val: bool) {
    DEBUG_MODE.with(|v| *v.borrow_mut() = val)
}
