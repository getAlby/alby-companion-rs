use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

use reqwest::header::HeaderMap;

use crate::{get_tor_password, get_tor_port, get_tor_username, write_debug};
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
    let mut url = match reqwest::Url::parse(&message.url) {
        Ok(u) => u,
        Err(err) => return Err(ReqError::Message(format!("Can not parse URL: {}", err)))
    };
    let client = match url.domain() {
        Some(host) => match host.contains(".onion") {
            true => {
                let proxy = reqwest::Proxy::all(&format!("socks5h://127.0.0.1:{}", get_tor_port()))?.basic_auth(&get_tor_username(), &get_tor_password());
                reqwest::blocking::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .timeout(Some(Duration::from_secs(15)))
                    .proxy(proxy)
                    .build()?
            },
            false => {
                reqwest::blocking::Client::builder()
                    .timeout(Some(Duration::from_secs(15)))
                    .build()?
            }
        },
        None => return Err(ReqError::Message("Can not parse domain".to_string())),
    };
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
