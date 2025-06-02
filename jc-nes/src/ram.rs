use crate::bus::Device;

pub struct Ram {
    pub mem: Vec<u8>,
}

impl Ram {
    pub fn new(mem: Vec<u8>) -> Ram {
        Ram { mem }
    }
}

impl Device for Ram {
    fn read(&mut self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
