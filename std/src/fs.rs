// use alloc::{string::String, vec::Vec};

// pub struct File {
//     path: Path,
// }

// impl File {
//     pub fn open(path: Path) -> File {
//         todo!();
//         File { path }
//     }

//     pub fn read(&self) -> &[u8] {
//         todo!()
//     }

//     pub fn write(&mut self, data: &[u8]) {
//         todo!()
//     }

//     pub fn close(&mut self) {
//         todo!()
//     }

//     pub fn append(&mut self, data: &[u8]) {
//         let old_data = self.read();

//         let mut new_data = Vec::with_capacity(old_data.len() + data.len());
//         new_data.extend_from_slice(old_data);
//         new_data.extend_from_slice(data);

//         self.write(&new_data);
//     }
// }

// pub struct Path {
//     path: String,
// }

// impl Path {
//     pub fn new(path: String) -> Path {
//         Path { path }
//     }
// }
