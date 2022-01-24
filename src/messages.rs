use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_json::Value as SerdeValue;

use crate::{is_debug_mode, write_debug};
use crate::requests::get_response;
use crate::tor::wait_for_tor;

#[derive(Deserialize, Debug)]
pub struct ReqMessage {
    pub id: String,
    pub url: String,
    pub method: String,
    pub body: Option<String>,
    pub params: Option<HashMap<String, String>>,
    pub headers: Option<HashMap<String, String>>,
    pub action: Option<String>,
    pub certificate: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ResMessage {
    pub id: String,
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
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
            action: None,
            certificate: None,
        }
    }
}

pub fn handler(v: SerdeValue) -> Result<ResMessage, String> {
    if is_debug_mode() {
        write_debug(format!("Incoming message: \n{:#?}", &v));
    } else {
        write_debug("Incoming message");
    }
    let msg: ReqMessage = match serde_json::from_value::<ReqMessage>(v) {
        Ok(m) => m,
        Err(err) => return Err(format!("Can not parse message: \n{:#?}", err))
    };
    let id = msg.id.clone();
    match get_response_msg(msg) {
        Ok(response) => {
            if is_debug_mode() {
                write_debug(format!("[{}]\t Outgoing message: \n{:#?}", &id, &response));
            } else {
                write_debug(format!("[{}]\t Outgoing message", &id));
            }
            Ok(response)
        },
        Err(err) => {
            write_debug(format!("[{}]\t Request handling error: \n{}", &id, &err));
            Ok(ResMessage {
                id,
                status: 500,
                body: err,
                headers: HashMap::from([("X-Alby-Internal".to_string(), "true".to_string())])
            })
        }
    }
}

fn get_response_msg(msg: ReqMessage) -> Result<ResMessage, String>
{
    if let Some(action) = &msg.action {
        if action == "startTor" {
            if crate::is_tor_started() {
                return Ok(get_tor_started_msg());
            }
            crate::tor::launch_tor();
            return if wait_for_tor(30, &crate::get_logfile_path()) {
                Ok(get_tor_started_msg())
            } else {
                Ok(get_tor_failed_start_msg())
            }
        }
    }
    match get_response(msg) {
        Ok(res) => Ok(res),
        Err(err) => Err(format!("Can not get response from resource: {:#?}", err))
    }
}

pub fn get_tor_failed_start_msg() -> ResMessage {
    ResMessage {
        id: "status".to_string(),
        status: 502,
        body: String::from("Can not launch Tor"),
        headers: HashMap::from([("X-Alby-Internal".to_string(), "true".to_string())]),
    }
}


pub fn get_tor_started_msg() -> ResMessage {
    ResMessage {
        id: "status".to_string(),
        status: 100,
        body: "tor_started".to_string(),
        headers: HashMap::from([("X-Alby-Internal".to_string(), "true".to_string())]),
    }
}

pub fn send_stdout_msg(msg: ResMessage) -> bool {
    chrome_native_messaging::send_message(std::io::stdout(), &msg).is_ok()
}
