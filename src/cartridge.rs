use super::bus::{BusRead, BusWrite};
use std::fs::File;
use std::io::Read;

pub struct Cartridge {
    pub prg_mem: Vec<u8>,
    pub char_mem: Vec<u8>,
}

impl Cartridge {
    pub fn new(size: usize) -> Cartridge {
        Cartridge {
            prg_mem: vec![0u8; size],
            char_mem: vec![0u8; size],
        }
    }

    pub fn from_path(path: &str) -> Cartridge {
        let mut file = File::open(path).unwrap();
        let mut rom = Vec::new();
        file.read_to_end(&mut rom).unwrap();

        // let mut prg_mem = vec![0u8; 16 * 1024]; // 16kB per bank
        // let mut char_mem = vec![0u8; 8 * 1024]; // 8kB per bank

        // skip header (16 bytes)
        let mut bytes = rom.bytes().skip(16);
        let prg_mem = bytes.by_ref().take(16 * 1024).flatten().collect();
        let char_mem = bytes.by_ref().take(8 * 1024).flatten().collect();

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
