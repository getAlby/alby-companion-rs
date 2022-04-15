use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::time::Duration;

use reqwest::header::HeaderMap;

use crate::{get_tor_password, get_tor_port, get_tor_username, is_debug_mode, write_debug};
use crate::messages::{ReqMessage, ResMessage};

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

pub fn get_response(message: ReqMessage) -> Result<ResMessage, ReqError> {
    if is_debug_mode() {
        write_debug(format!("message received: {:#?}", &message));
    } else {
        write_debug(format!("message received: {:#?}", &message.id));
    }
    let id = message.id.clone();
    let mut url = match reqwest::Url::parse(&message.url) {
        Ok(u) => u,
        Err(err) => return Err(ReqError::Message(format!("[{}]\t Can not parse URL: {}", &id, err)))
    };
    let is_clearnet = match url.domain() {
        Some(host) => !host.contains(".onion"),
        None => false,
    };
    if !is_clearnet {
        write_debug_about_msg("Sending this request using Tor", &id);
        if !crate::is_tor_started() {
            crate::tor::launch_tor();
        }
        if !crate::is_tor_ready() && !crate::tor::wait_for_tor(30, &crate::get_logfile_path()) {
            return Ok(crate::messages::get_tor_failed_start_msg())
        }
    }

    let mut builder = reqwest::blocking::Client::builder().timeout(Some(Duration::from_secs(75)));
    let mut cert_added = false;
    if let Some(cert_str) = message.certificate {
        if let Ok(cert_bytes) = base64::decode_config(&cert_str, base64::URL_SAFE) {
            if let Ok(cert) = reqwest::Certificate::from_der(&cert_bytes) {
                builder = builder.add_root_certificate(cert);
                cert_added = true;
            }
        } else if let Ok(cert_bytes) = base64::decode(&cert_str) {
            if let Ok(cert) = reqwest::Certificate::from_der(&cert_bytes) {
                builder = builder.add_root_certificate(cert);
                cert_added = true;
            }
        }
        if !cert_added {
            if let Ok(cert) = reqwest::Certificate::from_pem(cert_str.as_bytes()) {
                builder = builder.add_root_certificate(cert);
                cert_added = true;
            }
        }
    }
    if cert_added {
        write_debug_about_msg("Custom certificate has been set for this request", &id);
    }
    if !is_clearnet && !cert_added {
        builder = builder.danger_accept_invalid_certs(true);
    }
    if !is_clearnet {
        let proxy = reqwest::Proxy::all(&format!("socks5h://127.0.0.1:{}", get_tor_port()))?.basic_auth(&get_tor_username(), &get_tor_password());
        builder = builder.proxy(proxy);
    }

    let client = builder.build()?;

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
                write_debug_about_msg("headers were not parsed", &id);
                HeaderMap::new()
            }
        },
        None => HeaderMap::new(),
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
    let length = body.len();
    if is_debug_mode() {
        write_debug_about_msg(format!("server response status: {}, length: {} response: {:#?}", &status, &length, &body), &id);
    } else {
        write_debug_about_msg(format!("server response status: {}", &status), &id);
    }


    Ok(ResMessage {
        id: message.id,
        status: status.into(),
        body,
        headers: res_headers
    })
}

fn write_debug_about_msg<T: Display>(dbg: T, msg_id: &str) {
    write_debug(format!("[{}]\t {}", msg_id, dbg));
}
