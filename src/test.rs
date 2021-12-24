use std::{fs, thread};

use crate::SerdeValue;

#[test]
pub fn test_tor() {
    crate::prepare_log_file();
    crate::launch_tor();
    'a: for _ in 0..15 {
        let log = get_log();
        if log.contains(" Bootstrapped 100% (done): Done") {
            break 'a;
        }
        thread::sleep(std::time::Duration::from_secs(1));
    }

    match crate::get_response(SerdeValue::from(true)) {
        Ok(r) => assert!(!r.is_empty()),
        Err(e) => panic!("e: {:?}", e)
    }
    match crate::get_response(SerdeValue::from("")) {
        Ok(r) => assert!(!r.is_empty()),
        Err(e) => panic!("e: {:?}", e)
    }
}

fn get_log() -> String {
    match fs::read_to_string(crate::get_logfile_path()) {
        Ok(content) => content,
        Err(_) => String::new()
    }
}
