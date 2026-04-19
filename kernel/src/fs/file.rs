use alloc::boxed::Box;

pub trait File {
    fn read(&self) -> Box<[u8]>;

    fn write(&mut self, data: &[u8]);

    fn close(&mut self);

    fn name(&self) -> &str;
}
