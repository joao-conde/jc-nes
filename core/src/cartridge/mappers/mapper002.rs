use crate::bus::Device;

pub struct PRGMapper002 {
    mem: Vec<u8>,
    cur_bank_lo: u8,
    cur_bank_hi: u8,
}

impl PRGMapper002 {
    pub fn new(prg_rom: Vec<u8>, prg_banks: u8) -> PRGMapper002 {
        PRGMapper002 {
            mem: prg_rom,
            cur_bank_lo: 0x00,
            cur_bank_hi: prg_banks - 1,
        }
    }

    fn map_address(&self, address: u16) -> u16 {
        if address <= 0x3FFF {
            (self.cur_bank_lo as u16).wrapping_mul(0x4000).wrapping_add(address & 0x3FFF)
        } else if address > 0x3FFF {
            (self.cur_bank_hi as u16).wrapping_mul(0x4000).wrapping_add(address & 0x3FFF)
        } else {
            address
        }
    }
}

impl Device for PRGMapper002 {
    fn read(&mut self, address: u16) -> u8 {
        let address = self.map_address(address);
        self.mem[address as usize]
    }

    fn write(&mut self, _address: u16, data: u8) {
        self.cur_bank_lo = data;
    }
}

pub struct CHRMapper002 {
    mem: Vec<u8>,
}

impl CHRMapper002 {
    pub fn new(mem: Vec<u8>, banks: u8) -> CHRMapper002 {
        if banks == 0 {
            CHRMapper002 {
                mem: [0u8; 8 * 1024].to_vec(),
            }
        } else {
            CHRMapper002 { mem: mem }
        }
    }
}

impl Device for CHRMapper002 {
    fn read(&mut self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
