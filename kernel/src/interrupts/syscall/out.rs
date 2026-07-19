use crate::print;

pub fn out_handler(string: Option<&str>) -> i64 {
    match string {
        Some(string) => {
            print!("{string}");
            0
        }
        None => -1,
    }
}
