use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub fn print_block(block: &[u8]) -> String {
    let mut lines = Vec::new();
    for (i, line) in block.chunks(16).enumerate() {
        lines.push(format!(
            "{i:02x}0:  {}  |{}|",
            line.iter()
                .map(|x| format!("{x:02x}"))
                .collect::<Vec<String>>()
                .join(" "),
            line.iter()
                .map(|x| match *x {
                    0x20..=0x7e => char::from(*x),
                    _ => '.',
                })
                .collect::<String>()
        ));
    }
    lines.join("\n")
}
