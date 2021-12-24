use std::thread;

use crate::SerdeValue;

#[test]
pub fn test_tor() {
    crate::launch_tor();
    thread::sleep(std::time::Duration::from_secs(10));
    match crate::get_response(SerdeValue::from(true)) {
        Ok(r) => assert!(!r.is_empty()),
        Err(e) => panic!("e: {:?}", e)
    }
}
