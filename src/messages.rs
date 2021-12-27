use std::collections::HashMap;
use serde_json::Value as SerdeValue;
use crate::write_debug;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use crate::requests::get_response;


#[derive(Deserialize, Debug)]
pub struct ReqMessage {
    pub id: String,
    pub url: String,
    pub method: String,
    pub body: Option<String>,
    pub params: Option<HashMap<String, String>>,
    pub headers: Option<HashMap<String, String>>,
    pub action: Option<String>,
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
        }
    }
}

pub fn handler(v: SerdeValue) -> Result<ResMessage, String> {
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



pub fn send_tor_started_msg() {
    send_stdout_msg(ResMessage {
        id: "status".to_string(),
        status: 100,
        body: "tor_started".to_string(),
        headers: HashMap::from([("X-Alby-Internal".to_string(), "true".to_string())])
    });
}

pub fn send_stdout_msg(msg: ResMessage) -> bool {
    chrome_native_messaging::send_message(std::io::stdout(), &msg).is_ok()
}
