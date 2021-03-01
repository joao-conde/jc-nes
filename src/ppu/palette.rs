use crate::bus::Device;

pub struct Palette {
    mem: Vec<u8>,
}

impl Palette {
    pub fn new(mem: Vec<u8>) -> Palette {
        Palette { mem }
    }
}

impl Device for Palette {
    fn read(&mut self, address: u16) -> u8 {
        // let mut address = address & 0x001F;
        // if address == 0x0010 {
        //     address = 0x0000
        // }
        // if address == 0x0014 {
        //     address = 0x0004
        // }
        // if address == 0x0018 {
        //     address = 0x0008
        // }
        // if address == 0x001C {
        //     address = 0x000C
        // }

        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        // let mut address = address & 0x001F;
        // if address == 0x0010 {
        //     address = 0x0000
        // }
        // if address == 0x0014 {
        //     address = 0x0004
        // }
        // if address == 0x0018 {
        //     address = 0x0008
        // }
        // if address == 0x001C {
        //     address = 0x000C
        // }

        self.mem[address as usize] = data;
    }
}
