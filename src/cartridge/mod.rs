pub mod mappers;

use std::{fs::File, io::Read};

#[derive(Default)]
pub struct Cartridge {
    pub(in crate::cartridge) prg_rom: Vec<u8>,
    pub(in crate::cartridge) chr_rom: Vec<u8>,
    pub(in crate) meta: Meta,
}

#[derive(Clone, Default)]
pub struct Meta {
    pub(in crate) mapper_id: u8,
    pub(in crate) name: String,
    pub(in crate) mirror: bool,
    pub(in crate) prg_banks: u8,
    pub(in crate) chr_banks: u8,
}

impl Cartridge {
    pub fn new(path: &str) -> Cartridge {
        let mut cartridge = Cartridge::default();

        // read ROM bytes
        let mut file = File::open(path).unwrap();
        let mut rom = Vec::new();
        file.read_to_end(&mut rom).unwrap();

        let mut bytes = rom.bytes();

        cartridge.meta.name =
            String::from_utf8(bytes.by_ref().take(4).flatten().collect()).unwrap();
        cartridge.meta.prg_banks = bytes.by_ref().next().unwrap().unwrap();
        cartridge.meta.chr_banks = bytes.by_ref().next().unwrap().unwrap();

        let mapper1 = bytes.by_ref().next().unwrap().unwrap();
        let mapper2 = bytes.by_ref().next().unwrap().unwrap();

        let _prg_ram_len = bytes.by_ref().next().unwrap().unwrap();

        let _tv_system1 = bytes.by_ref().next().unwrap().unwrap();
        let _tv_system2 = bytes.by_ref().next().unwrap().unwrap();

        let _unused = bytes.by_ref().take(5).flatten().collect::<Vec<u8>>();

        // ff a "trainer" exists
        if (mapper1 & 0x04) >> 2 == 1 {
            let _trainer = bytes.by_ref().take(512).flatten().collect::<Vec<u8>>();
        }

        cartridge.meta.mapper_id = ((mapper2 >> 4) << 4) | (mapper1 >> 4);
        cartridge.meta.mirror = mapper1 & 0x01 == 1;

        let file_type = 1; // TODO not hard-code (works for DK and nestest)
        match file_type {
            1 => {
                let prg_len = cartridge.meta.prg_banks as usize * 16 * 1024;
                cartridge.prg_rom.resize(prg_len, 0);

                let _mp1 = 0xFFF9 & 0x3FFF;
                let _mp2 = 0xFFFF & 0x3FFF;

                let prg_rom = bytes.by_ref().take(prg_len).flatten().collect::<Vec<u8>>();
                cartridge.prg_rom.copy_from_slice(&prg_rom);

                let chr_len = cartridge.meta.chr_banks as usize * 8 * 1024;
                cartridge.chr_rom.resize(chr_len, 0);
                cartridge.chr_rom = bytes.by_ref().take(chr_len).flatten().collect();
            }
            _ => panic!("unknown file type"),
        }

        cartridge
    }
}
