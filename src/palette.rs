use crate::bus::{BusRead, BusWrite};

pub struct Palette {
    mem: Vec<u8>,
}

impl Palette {
    pub fn new(mem: Vec<u8>) -> Palette {
        Palette { mem }
    }
}

impl BusRead for Palette {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }
}

impl BusWrite for Palette {
    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
