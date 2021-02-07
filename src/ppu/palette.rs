use crate::bus::{BusRead, BusWrite};

pub struct Palette {
    mem: [u8; 256],
}

impl Palette {
    pub fn new() -> Palette {
        Palette { mem: [0u8; 256] }
    }
}

impl BusRead for Palette {
    fn read(&mut self, address: u16) -> u8 {
        self.mem[address as usize]
    }
}

impl BusWrite for Palette {
    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
