use super::bus::{BusRead, BusWrite};
use std::fs::File;
use std::io::Read;

pub struct Cartridge {
    prg_mem: Vec<u8>,
    char_mem: Vec<u8>,
}

impl Cartridge {
    pub fn from_path(path: &str) -> Cartridge {
        let mut file = File::open(path).unwrap();
        let mut rom = Vec::new();
        file.read_to_end(&mut rom).unwrap();

        // skip header (16 bytes)
        let mut bytes = rom.bytes().skip(16);
        let prg_mem = bytes.by_ref().take(16 * 1024).flatten().collect(); // 16kB per bank
        let char_mem = bytes.by_ref().take(8 * 1024).flatten().collect(); // 8kB per bank

        Cartridge { prg_mem, char_mem }
    }
}

impl BusRead for Cartridge {
    fn read(&self, address: u16) -> u8 {
        self.prg_mem[address as usize]
    }
}

impl BusWrite for Cartridge {
    fn write(&mut self, address: u16, data: u8) {
        self.prg_mem[address as usize] = data;
    }
}
