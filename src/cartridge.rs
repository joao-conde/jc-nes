use std::{fs::File, io::Read};

use crate::bus::{BusRead, BusWrite};

pub struct Cartridge {
    prg_rom: Vec<u8>,
    pattern_tables: Vec<u8>,
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

        let pattern_tables = bytes.by_ref().take(8 * 1024).flatten().collect::<Vec<u8>>(); // 8kB per bank

        Cartridge {
            prg_rom,
            pattern_tables,
        }
    }
}

impl BusRead for Cartridge {
    fn read(&self, address: u16) -> u8 {
        self.pattern_tables[address as usize]
    }
}

impl BusWrite for Cartridge {
    fn write(&mut self, address: u16, data: u8) {
        self.prg_rom[address as usize] = data; // TODO mapper intercept
    }
}
