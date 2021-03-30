pub mod mappers;

use std::{fs::File, io::Read};

pub struct Cartridge {
    pub(in crate) prg_rom: Vec<u8>,
    pub(in crate) chr_rom: Vec<u8>,
    pub(in crate) mapper_id: u8,
    pub(in crate) prg_banks: u8,
    pub(in crate) chr_banks: u8,
    pub(in crate) mirror: MirrorMode,
}

pub enum MirrorMode {
    Horizontal,
    Vertical,
}

impl Cartridge {
    pub fn new(path: &str) -> Cartridge {
        let mut file = File::open(path).unwrap();
        let mut rom = Vec::new();
        file.read_to_end(&mut rom).unwrap();

        let mut bytes = rom.bytes();

        // iNES initial 4 bytes "NES<EOF>"
        let nes_signature = bytes.by_ref().take(4).flatten().collect::<Vec<u8>>();
        assert!(nes_signature == [0x4E, 0x45, 0x53, 0x1A]);

        let prg_banks = bytes.by_ref().next().unwrap().unwrap();
        let chr_banks = bytes.by_ref().next().unwrap().unwrap();

        let flags6 = bytes.by_ref().next().unwrap().unwrap();
        let flags7 = bytes.by_ref().next().unwrap().unwrap();

        let _prg_ram_len = bytes.by_ref().next().unwrap().unwrap();
        let _flags9 = bytes.by_ref().next().unwrap().unwrap();
        let _flags10 = bytes.by_ref().next().unwrap().unwrap();

        let _unused = bytes.by_ref().take(5).flatten().collect::<Vec<u8>>();

        // if a "trainer" exists
        if (flags6 & 0x04) >> 2 == 1 {
            let _trainer = bytes.by_ref().take(512).flatten().collect::<Vec<u8>>();
        }

        let mapper_id = ((flags7 >> 4) << 4) | (flags6 >> 4);
        let mirror = if flags6 & 0x01 == 1 {
            MirrorMode::Vertical
        } else {
            MirrorMode::Horizontal
        };

        // TODO find other formats
        let file_type = 1;
        let (prg_rom, chr_rom) = match file_type {
            1 => {
                let prg_len = prg_banks as usize * 16 * 1024;
                let prg_rom = bytes.by_ref().take(prg_len).flatten().collect::<Vec<u8>>();

                let chr_len = chr_banks as usize * 8 * 1024;
                let chr_rom = bytes.by_ref().take(chr_len).flatten().collect();

                (prg_rom, chr_rom)
            }
            _ => panic!("unknown file type"),
        };

        Cartridge {
            prg_rom,
            chr_rom,
            mapper_id,
            prg_banks,
            chr_banks,
            mirror,
        }
    }
}
