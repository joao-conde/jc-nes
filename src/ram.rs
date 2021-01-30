use super::bus::{BusRead, BusWrite};

pub struct RAM {
    pub mem: Vec<u8>,
}

impl RAM {
    pub fn new(size: usize) -> RAM {
        RAM {
            mem: vec![0u8; size],
        }
    }
}

impl BusRead for RAM {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }
}

impl BusWrite for RAM {
    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
