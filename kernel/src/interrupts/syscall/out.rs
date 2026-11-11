use crate::print;

pub fn out_handler(string: Result<&str, ()>) -> i64 {
    match string {
        Ok(string) => {
            print!("{string}");
            0
        }
        Err(()) => -1,
    }
}
