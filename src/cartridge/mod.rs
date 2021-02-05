pub mod mappers;

use std::{fs::File, io::Read};

#[derive(Default)]
pub struct Cartridge {
    pub(in crate::cartridge) prg_rom: Vec<u8>,
    pub(in crate::cartridge) chr_rom: Vec<u8>,
    pub(in crate) header: Header,
}

#[derive(Clone, Copy, Default)]
pub struct Header {
    pub(in crate) mapper_id: u8,
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

        let header = Header { mapper_id: 0 };

        Cartridge {
            prg_rom,
            chr_rom,
            header,
        }
    }
}
