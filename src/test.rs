use std::fs;

use serial_test::serial;

use crate::{get_log, get_logfile_path};
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
pub fn test_logging_messages_only_in_debug_mode() {
    crate::prepare_log_file();

    let mut msg: ReqMessage = Default::default();
    msg.id = "14".to_string();
    msg.url = String::from("https://github.com");
    msg.body = Some(String::from("pa_zz_word"));

    match crate::requests::get_response(msg) {
        Ok(r) => {
            assert_eq!(r.id, String::from("14"));
            let log = get_log(&get_logfile_path());
            assert!(!log.contains("pa_zz_word"));
        },
        Err(e) => panic!("e: {:#?}", e)
    }

    let mut msg: ReqMessage = Default::default();
    msg.id = "15".to_string();
    msg.url = String::from("https://github.com");
    msg.body = Some(String::from("pa_zz_word"));
    crate::set_debug_mode(true);

    match crate::requests::get_response(msg) {
        Ok(r) => {
            assert_eq!(r.id, String::from("15"));
            let log = get_log(&get_logfile_path());
            assert!(log.contains("pa_zz_word"));
        },
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

#[test]
#[serial]
#[ignore]
pub fn test_der_cert() {
    let cert_str = String::from("MIICJjCCAc2gAwIBAgIQHRYicyiRtp-G-uZNJtaGnTAKBggqhkjOPQQDAjA5MR8wHQYDVQQKExZsbmQgYXV0b2dlbmVyYXRlZCBjZXJ0MRYwFAYDVQQDEw1saW9vcHRpLmxvY2FsMB4XDTIxMTAyMTExMDYwMloXDTIyMTIxNjExMDYwMlowOTEfMB0GA1UEChMWbG5kIGF1dG9nZW5lcmF0ZWQgY2VydDEWMBQGA1UEAxMNbGlvb3B0aS5sb2NhbDBZMBMGByqGSM49AgEGCCqGSM49AwEHA0IABMuzk_4We3TjwXwe-gTAqM8unTW9Vdu5MDPlnN7ii7-Nfk_oadIJLd_6MUGUjIpsjr1AcBg5iY-wDLPYjA-xAtejgbYwgbMwDgYDVR0PAQH_BAQDAgKkMBMGA1UdJQQMMAoGCCsGAQUFBwMBMA8GA1UdEwEB_wQFMAMBAf8wHQYDVR0OBBYEFGpUZqHMoSMF_pY_wWbCeql_Tap0MFwGA1UdEQRVMFOCCWxvY2FsaG9zdIINbGlvb3B0aS5sb2NhbIIEdW5peIIKdW5peHBhY2tldIIHYnVmY29ubocEfwAAAYcQAAAAAAAAAAAAAAAAAAAAAYcEChUVCTAKBggqhkjOPQQDAgNHADBEAiAllMVRefoE_jA6u5yqA5vlD0wBL1P2cTjwmeKlkJPlwgIgFa_BAAroLE1HGNZkj5I2fAvYWIegE5c_LHXoWbzKgtw");
    if let Err(e) = base64::decode_config(&cert_str, base64::URL_SAFE) {
        panic!("can not decode cert: {:#?}", e)
    }

    crate::prepare_log_file();
    let mut msg: ReqMessage = Default::default();
    msg.id = "13".to_string();
    msg.certificate = Some(cert_str);
    msg.url = String::from("https://wqskhzt3oiz76dgqbqh27j3qw5aeaui3jxyexzuwxqa5czzo24i3z3ad.onion:8080");
    match crate::requests::get_response(msg) {
        Ok(r) => assert_eq!(r.id, String::from("13")),
        Err(e) => panic!("e: {:#?}", e)
    }
}

#[test]
#[serial]
pub fn test_create_lock_file() {
    use crate::{create_lock_file, get_lock_file_path};

    let path = get_lock_file_path();
    assert!(!path.is_empty());
    let _ = fs::write(&path, "12345");
    // scope for RAII
    {
        let lock1 = create_lock_file();
        assert!(lock1.is_some());
        let lock2 = create_lock_file();
        assert!(lock2.is_none());
    }
    // lock1 and lock2 should call their "Drop" and remove their files
    let lock3 = create_lock_file();
    assert!(lock3.is_some());
    // non-parseable PID is a reason to remove lock 
    let _ = fs::write(&path, "---{!}---");
    let lock4 = create_lock_file();
    assert!(lock4.is_some());
}
