use super::bus::{Read, Write};

pub struct Cartridge {
    pub mem: Vec<u8>,
}

impl Cartridge {
    pub fn new(size: usize) -> Cartridge {
        Cartridge { mem: vec![0u8; size] }
    }
}

impl Read for Cartridge {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }
}

impl Write for Cartridge {
    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
