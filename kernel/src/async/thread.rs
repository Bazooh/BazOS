pub struct Thread {}

impl Thread {
    pub fn new(entry_point: fn()) -> Thread {
        Thread {}
    }

    pub fn checkpoint(&mut self) {
        todo!()
    }

    pub fn restore(&self) {
        todo!()
    }
}
