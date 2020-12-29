use super::bus::Device;
pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    pc: u8,
    sp: u8,
    sr: u8,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            pc: 0x00,
            sp: 0x00,
            sr: 0x00,
        }
    }
}

impl Device for CPU {
    fn read(&self, address: usize) -> usize {
        println!("CPU reading from {:0x}!", address);
        self.a as usize
    }

    fn write(&self, address: usize) {
        println!("CPU writing to {:0x}", address)
    }
}
