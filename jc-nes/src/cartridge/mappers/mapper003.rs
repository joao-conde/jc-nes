use crate::bus::Device;
use crate::cartridge::Cartridge;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct PrgMapper {
    cur_bank: Rc<RefCell<usize>>,
    prg_mem: Vec<u8>,
    prg_banks: usize,
}

#[derive(Clone)]
pub struct ChrMapper {
    cur_bank: Rc<RefCell<usize>>,
    chr_mem: Vec<u8>,
}

pub fn new_mapper(cartridge: Cartridge) -> (PrgMapper, ChrMapper) {
    let cur_bank = Rc::new(RefCell::new(0));

    let prg_mapper = PrgMapper {
        cur_bank: cur_bank.clone(),
        prg_mem: cartridge.prg_rom,
        prg_banks: cartridge.prg_banks,
    };
    let chr_mapper = ChrMapper {
        cur_bank,
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

    fn write(&mut self, _address: u16, data: u8) {
        self.cur_bank.replace((data & 0x03) as usize);
    }
}

impl Device for ChrMapper {
    fn read(&mut self, address: u16) -> u8 {
        let address = *self.cur_bank.borrow() * 0x2000 + address as usize;
        self.chr_mem[address]
    }

    fn write(&mut self, _address: u16, _data: u8) {}
}
