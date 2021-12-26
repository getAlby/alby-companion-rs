use crate::ReqMessage;

#[test]
pub fn test_tor() {
    crate::prepare_log_file();
    crate::launch_tor();
    let _ = crate::wait_for_tor(15, &crate::get_logfile_path());

    match crate::get_response(ReqMessage {
        id: "11".to_string(),
        url: String::from("https://facebookwkhpilnemxj7asaniu7vnjjbiltxjqhye3mhbshg7kx5tfyd.onion"),
        method: "GET".to_string(),
        body: None,
        params: None,
        headers: None
    }) {
        Ok(r) => assert_eq!(r.id, String::from("11")),
        Err(e) => panic!("e: {:?}", e)
    }
    match crate::get_response(ReqMessage {
        id: "12".to_string(),
        url: String::from("https://wqskhzt3oiz76dgqbqh27j3qw5aeaui3jxyexzuwxqa5czzo24i3z3ad.onion:8080"),
        method: "GET".to_string(),
        body: None,
        params: None,
        headers: None
    }) {
        Ok(r) => assert_eq!(r.id, String::from("12")),
        Err(e) => panic!("e: {:?}", e)
    }
}
