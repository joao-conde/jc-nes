use super::bus::{Read, Write};

pub struct NameTable {
    pub mem: Vec<u8>,
}

impl NameTable {
    pub fn new(size: usize) -> NameTable {
        NameTable { mem: vec![0u8; size] }
    }
}

impl Read for NameTable {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }
}

impl Write for NameTable {
    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
