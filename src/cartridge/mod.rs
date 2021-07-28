pub mod mappers;

pub struct Cartridge {
    pub(in crate) prg_rom: Vec<u8>,
    pub(in crate) chr_rom: Vec<u8>,
    pub(in crate) mapper_id: usize,
    pub(in crate) prg_banks: usize,
    pub(in crate) chr_banks: usize,
    pub(in crate) mirror: MirrorMode,
}

#[derive(Clone, Copy)]
pub enum MirrorMode {
    OneScreenLo,
    OneScreenHi,
    Horizontal,
    Vertical,
}

impl Cartridge {
    pub fn new(rom: &[u8]) -> Cartridge {
        // iNES initial 4 bytes "NES<EOF>"
        let nes_signature = &rom[0..4];
        assert!(
            nes_signature == [0x4E, 0x45, 0x53, 0x1A],
            "Not a .NES file (header signature not correct)"
        );

        let prg_banks = rom[4] as usize;
        let chr_banks = rom[5] as usize;

        let flag6 = rom[6];
        let flag7 = rom[7];

        let has_trainer = (flag6 & 0x04) >> 2 == 1;

        let mapper_id = (((flag7 >> 4) << 4) | (flag6 >> 4)) as usize;
        let mirror = if flag6 & 0x01 == 1 {
            MirrorMode::Vertical
        } else {
            MirrorMode::Horizontal
        };

        let file_type = if flag7 & 0x0C == 0x08 { 2 } else { 1 };
        let (prg_rom, chr_rom) = match file_type {
            1 => {
                let index = 16 + if has_trainer { 512 } else { 0 };

                let prg_len = prg_banks as usize * 16 * 1024;
                let prg_rom = rom[index..index + prg_len].to_vec();

                let chr_len = chr_banks as usize * 8 * 1024;
                let chr_rom = rom[index + prg_len..index + prg_len + chr_len].to_vec();

                (prg_rom, chr_rom)
            }
            _ => panic!("Unknown file type {}", file_type),
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
