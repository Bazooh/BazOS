use core::str::from_utf8;

use crate::{eprintln, print};

pub fn out_handler(ptr: *const u8, len: usize) -> isize {
    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
    return match from_utf8(bytes) {
        Ok(string) => {
            print!("{string}");
            0
        }
        Err(err) => {
            eprintln!("{err}");
            -1
        }
    };
}
