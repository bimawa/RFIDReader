use core::fmt::Write;
use heapless::String;

pub fn generate_filename(uid: &[u8; 8]) -> String<12> {
    let mut name: String<12> = String::new();
    let _ = write!(name, "{:02X}{:02X}.DMP", uid[0], uid[1]);
    name
}
