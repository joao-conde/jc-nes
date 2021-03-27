use crate::bus::Device;

pub struct PRGMapper000 {
    mem: Vec<u8>,
    banks: u8,
}

impl PRGMapper000 {
    pub fn new(prg_rom: Vec<u8>, prg_banks: u8) -> PRGMapper000 {
        PRGMapper000 {
            mem: prg_rom,
            banks: prg_banks,
        }
    }
}

impl Device for PRGMapper000 {
    fn read(&mut self, address: u16) -> u8 {
        let address = if self.banks == 1 {
            address & 0x3FFF
        } else {
            address
        };

        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        let address = if self.banks == 1 {
            address & 0x3FFF
        } else {
            address
        };

        self.mem[address as usize] = data;
    }
}

pub struct CHRMapper000 {
    mem: Vec<u8>,
}

impl CHRMapper000 {
    pub fn new(mem: Vec<u8>, banks: u8) -> CHRMapper000 {
        if banks == 0 {
            CHRMapper000 {
                mem: [0u8; 8 * 1024].to_vec(),
            }
        } else {
            CHRMapper000 { mem: mem }
        }
    }
}

impl Device for CHRMapper000 {
    fn read(&mut self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
