use super::Mapper;

// DK SPECIFIC
struct Mapper0 {
    prg_banks: u8,
    chr_banks: u8,
}

impl Mapper0 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Mapper0 {
        Mapper0 {
            prg_banks,
            chr_banks,
        }
    }
}

impl Mapper for Mapper0 {
    fn map_cpu_read(address: u16) -> u16 {
        todo!()
    }

    fn map_cpu_write(address: u16) -> u16 {
        todo!()
    }

    fn map_ppu_read(address: u16) -> u16 {
        todo!()
    }

    fn map_ppu_write(address: u16) -> u16 {
        todo!()
    }
}
