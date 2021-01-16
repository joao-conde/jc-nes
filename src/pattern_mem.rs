use super::bus::{Read, Write};

pub struct PatternMem {
    pub mem: Vec<u8>,
}

impl PatternMem {
    pub fn new(size: usize) -> PatternMem {
        PatternMem { mem: vec![0u8; size] }
    }
}

impl Read for PatternMem {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }
}

impl Write for PatternMem {
    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
