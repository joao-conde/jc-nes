use crate::bus::{BusRead, BusWrite};

pub struct NameTable {
    mem: Vec<u8>,
}

impl NameTable {
    pub fn new(mem: Vec<u8>) -> NameTable {
        NameTable { mem }
    }
}

impl BusRead for NameTable {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }
}

impl BusWrite for NameTable {
    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
