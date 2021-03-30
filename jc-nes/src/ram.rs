use crate::bus::Device;

pub struct RAM {
    mem: Vec<u8>,
}

impl RAM {
    pub fn new(mem: Vec<u8>) -> RAM {
        RAM { mem }
    }
}

impl Device for RAM {
    fn read(&mut self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
