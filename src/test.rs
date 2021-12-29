use serial_test::serial;

use crate::messages::ReqMessage;

#[test]
#[serial]
pub fn test_clearnet_request() {
    crate::prepare_log_file();
    let mut msg: ReqMessage = Default::default();
    msg.id = "10".to_string();
    msg.url = String::from("https://github.com");

    match crate::requests::get_response(msg) {
        Ok(r) => assert_eq!(r.id, String::from("10")),
        Err(e) => panic!("e: {:#?}", e)
    }
}

#[test]
#[serial]
pub fn test_tor_request() {
    crate::prepare_log_file();
    let mut msg: ReqMessage = Default::default();
    msg.id = "11".to_string();
    msg.url = String::from("https://facebookwkhpilnemxj7asaniu7vnjjbiltxjqhye3mhbshg7kx5tfyd.onion");
    match crate::requests::get_response(msg) {
        Ok(r) => assert_eq!(r.id, String::from("11")),
        Err(e) => panic!("e: {:#?}", e)
    }

    let mut msg: ReqMessage = Default::default();
    msg.id = "12".to_string();
    msg.url = String::from("https://wqskhzt3oiz76dgqbqh27j3qw5aeaui3jxyexzuwxqa5czzo24i3z3ad.onion:8080");

    match crate::requests::get_response(msg) {
        Ok(r) => assert_eq!(r.id, String::from("12")),
        Err(e) => panic!("e: {:#?}", e)
    }
}

#[test]
pub fn test_cli() {
    let opts = crate::cli::get_cli_options(crate::cli::get_args_from_string("command string --log_file=LF -t=TD"));
    assert_eq!(opts.log_file, Some(String::from("LF")));
    assert_eq!(opts.tor_dir, Some(String::from("TD")));
    let opts = crate::cli::get_cli_options(crate::cli::get_args_from_string("-l=LF"));
    assert_eq!(opts.log_file, Some(String::from("LF")));
    assert_eq!(opts.tor_dir, None);
}
