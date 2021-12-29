use std::collections::HashMap;
use std::thread;

use libtor::{LogDestination, LogLevel, Tor, TorFlag};

use crate::{exit, get_lock_file_path, get_log, get_logfile_path, get_tor_dir_path, get_tor_password, get_tor_port, get_tor_username, write_debug, write_debug_to};
use crate::messages::{ResMessage, send_stdout_msg};

pub fn launch_tor() {
    if crate::is_tor_started() {
        return;
    }
    crate::set_tor_is_started(true); // otherwise it will be possible to launch 2 starting processes
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

pub fn wait_for_tor(seconds: u8, log_file: &str) -> bool {
    let pid = crate::get_pid_key();
    for _ in 0..seconds {
        let log = get_log(log_file);
        if log.contains("Bootstrapped 100% (done):") && log.contains(&pid) {
            crate::set_tor_is_ready(true);
            return true;
        }
        thread::sleep(std::time::Duration::from_secs(1));
    }
    false
}
