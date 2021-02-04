use std::{fs::File, io::Read};

use crate::bus::BusRead;

pub struct Cartridge {
    prg_rom: [u8; 32 * 1024],
    pattern_table: [u8; 8 * 1024],
}

impl Cartridge {
    pub fn load_rom(path: &str) -> Cartridge {
        let mut cartridge = Cartridge::default();

        // read ROM bytes
        let mut file = File::open(path).unwrap();
        let mut rom = Vec::new();
        file.read_to_end(&mut rom).unwrap();

        // skip header (16 bytes) TODO parse header actually
        let mut bytes = rom.bytes().skip(16);

        // TODO actually get nbanks from header * 16kb per bank
        let prg_mem = bytes
            .by_ref()
            .take(32 * 1024)
            .flatten()
            .collect::<Vec<u8>>(); // 16kB per bank

        let char_mem = bytes.by_ref().take(8 * 1024).flatten().collect::<Vec<u8>>(); // 8kB per bank

        cartridge.prg_rom.clone_from_slice(&prg_mem);
        cartridge.pattern_table.clone_from_slice(&char_mem);

        cartridge
    }
}

impl Default for Cartridge {
    fn default() -> Self {
        Cartridge {
            prg_rom: [0u8; 32 * 1024],
            pattern_table: [0u8; 8 * 1024],
        }
    }
}

impl BusRead for Cartridge {
    fn read(&self, address: u16) -> u8 {
        self.pattern_table[address as usize]
    }
}
