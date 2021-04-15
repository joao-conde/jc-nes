use crate::bus::{Device, SharedMut};
use crate::cartridge::Cartridge;
use std::{cell::RefCell, rc::Rc};

pub struct PrgMapper {
	prg_bank_16_lo: usize,
	prg_bank_16_hi: usize,
	prg_bank_32: usize,
    prg_mem: Vec<u8>,

	control: SharedMut<u8>,
	load: u8,
	load_cnt: usize,
}

pub struct ChrMapper {
    control: SharedMut<u8>,
    chr_bank_4_lo: usize,
	chr_bank_4_hi: usize,
	chr_bank_8: usize,
    chr_mem: Vec<u8>,
}

pub fn new_mapper(cartridge: Cartridge) -> (PrgMapper, ChrMapper) {
    let control = Rc::new(RefCell::new(0x1C)); //TODO for sure?

    let prg_mapper = PrgMapper {
        prg_bank_16_lo: 0,
        prg_bank_16_hi: cartridge.prg_banks - 1,
        prg_bank_32: 0,
        prg_mem: cartridge.prg_rom,

        control: control.clone(),
        load: 0x00,
        load_cnt: 0,
    };
    let chr_mapper = ChrMapper {
        control: control,
        chr_bank_4_lo: 0,
        chr_bank_4_hi: 0,
        chr_bank_8: 0,
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
        if *self.control.borrow() & 0b10000 != 0 {
            // 4K CHR Bank Mode
            if address <= 0x3FFF {
                self.prg_bank_16_lo as u16 * 0x4000 + (address & 0x3FFF)
            }
            else {
                self.prg_bank_16_hi as u16 * 0x4000 + (address & 0x3FFF)
            }
        }
        else {
            // 8K CHR Bank Mode
            self.prg_bank_32 as u16 * 0x8000 + (address & 0x7FFF)
        }
    }  
}

impl ChrMapper {
    fn map_address(&self, address: u16) -> u16 {
        if *self.control.borrow() & 0b10000 != 0 {
            // 4K CHR Bank Mode
            if address <= 0x0FFF {
                self.chr_bank_4_lo as u16 * 0x1000 + (address & 0x0FFF)
            }
            else {
                self.chr_bank_4_hi as u16 * 0x1000 + (address & 0x0FFF)
            }
        }
        else {
            // 8K CHR Bank Mode
            self.chr_bank_8 as u16 * 0x2000 + (address & 0x1FFF)
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
        let address = self.map_address(address);
        self.chr_mem[address as usize]
    }

    fn write(&mut self, _address: u16, _data: u8) {}
}
