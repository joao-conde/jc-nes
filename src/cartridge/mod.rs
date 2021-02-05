pub mod mappers;

use std::{fs::File, io::Read};

pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl Cartridge {
    pub fn load_rom(path: &str) -> Cartridge {
        // read ROM bytes
        let mut file = File::open(path).unwrap();
        let mut rom = Vec::new();
        file.read_to_end(&mut rom).unwrap();

        // skip header (16 bytes) TODO parse header actually
        let mut bytes = rom.bytes().skip(16);

        // TODO actually get nbanks from header * 16kb per bank
        let prg_rom = bytes
            .by_ref()
            .take(32 * 1024) // DK SPECIFIC
            .flatten()
            .collect::<Vec<u8>>(); // 16kB per bank

        let chr_rom = bytes.by_ref().take(8 * 1024).flatten().collect::<Vec<u8>>(); // 8kB per bank

        Cartridge { prg_rom, chr_rom }
    }

    pub fn read_prg_rom(&self, address: u16) -> u8 {
        self.prg_rom[address as usize]
    }

    pub fn read_chr_rom(&self, address: u16) -> u8 {
        self.chr_rom[address as usize]
    }

    pub fn write_prg_rom(&mut self, address: u16, data: u8) {
        self.prg_rom[address as usize] = data;
    }
}
