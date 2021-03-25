use crate::bus::Device;

pub struct PRGMapper000 {
    prg_rom: Vec<u8>,
    prg_banks: u8,
}

impl PRGMapper000 {
    pub fn new(prg_rom: Vec<u8>, prg_banks: u8) -> PRGMapper000 {
        PRGMapper000 { prg_rom, prg_banks }
    }
}

impl Device for PRGMapper000 {
    fn read(&mut self, address: u16) -> u8 {
        let address = if self.prg_banks == 1 {
            address & 0x3FFF
        } else {
            address
        };

        self.prg_rom[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        let address = if self.prg_banks == 1 {
            address & 0x3FFF
        } else {
            address
        };

        self.prg_rom[address as usize] = data;
    }
}

pub struct CHRMapper000 {
    chr_rom: Vec<u8>,
}

impl CHRMapper000 {
    pub fn new(chr_rom: Vec<u8>) -> CHRMapper000 {
        CHRMapper000 { chr_rom }
    }
}

impl Device for CHRMapper000 {
    fn read(&mut self, address: u16) -> u8 {
        self.chr_rom[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.chr_rom[address as usize] = data;
    }
}
