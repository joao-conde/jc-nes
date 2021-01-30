use super::bus::{Read, Write};

pub struct Cartridge {
    pub prg_mem: Vec<u8>,
    pub char_mem: Vec<u8>,
}

impl Cartridge {
    pub fn new(size: usize) -> Cartridge {
        Cartridge { prg_mem: vec![0u8; size], char_mem: vec![0u8; size] }
    }

    pub fn from_rom(rom: &[u8]) -> Cartridge {
        let mut char_mem = vec![0u8; 64 * 1024]; // TODO  what size?
        let mut prg_mem = vec![0u8; 64 * 1024]; // TODO  what size?
        
        for i in 0..(16 * 1024) {
            prg_mem[0xC000 + i] = rom[i + 0x10];
        }

        for i in 0..(8 * 1024) {
            char_mem[i] = rom[i];
        }

        Cartridge { prg_mem, char_mem }
    }
}

impl Read for Cartridge {
    fn read(&self, address: u16) -> u8 {
        self.prg_mem[address as usize]
    }
}

impl Write for Cartridge {
    fn write(&mut self, address: u16, data: u8) {
        self.prg_mem[address as usize] = data;
    }
}
