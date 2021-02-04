#[derive(Default)]
pub struct Bus {
    mem: Vec<u8>,
}

impl Bus {
    pub fn new(mem: Vec<u8>) -> Bus {
        Bus { mem }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
