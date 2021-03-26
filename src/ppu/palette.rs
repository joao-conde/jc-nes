use crate::bus::Device;

pub struct Palette {
    mem: [u8; 256],
}

impl Palette {
    pub fn new() -> Palette {
        Palette { mem: [0u8; 256] }
    }

    // http://forums.nesdev.com/viewtopic.php?t=7719
    fn mirror(&self, address: u16) -> usize {
        (match address {
            0x0010 => 0x0000,
            0x0014 => 0x0004,
            0x0018 => 0x0008,
            0x001C => 0x000C,
            address => address
        }) as usize
    }
}

impl Device for Palette {
    fn read(&mut self, address: u16) -> u8 {
        let address = self.mirror(address);
        self.mem[address]
    }

    fn write(&mut self, address: u16, data: u8) {
        let address = self.mirror(address);
        self.mem[address] = data;
    }
}
