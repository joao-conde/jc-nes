use crate::bus::Device;
use crate::cartridge::Cartridge;

pub struct PrgMapper {
    prg_banks: usize,
    prg_mem: Vec<u8>,
}

pub struct ChrMapper {
    chr_mem: Vec<u8>,
}

pub fn new_mapper(cartridge: Cartridge) -> (PrgMapper, ChrMapper) {
    let prg_mapper = PrgMapper {
        prg_mem: cartridge.prg_rom,
        prg_banks: cartridge.prg_banks,
    };
    let chr_mapper = ChrMapper {
        chr_mem: if cartridge.chr_banks == 0 {
            [0u8; 8 * 1024].to_vec()
        } else {
            cartridge.chr_rom
        },
    };
    (prg_mapper, chr_mapper)
}

impl PrgMapper {
    fn map_address(&self, address: u16) -> u16 {
        if self.prg_banks == 1 {
            address & 0x3FFF
        } else {
            address & 0x7FFF
        }
    }
}

impl Device for PrgMapper {
    fn read(&mut self, address: u16) -> u8 {
        let address = self.map_address(address);
        self.prg_mem[address as usize]
    }

    fn write(&mut self, _address: u16, _data: u8) {}
}

impl Device for ChrMapper {
    fn read(&mut self, address: u16) -> u8 {
        self.chr_mem[address as usize]
    }

    fn write(&mut self, _address: u16, _data: u8) {}
}
