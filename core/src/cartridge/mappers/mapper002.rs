use crate::bus::{Device, SharedMut};
use crate::cartridge::Cartridge;
use std::{cell::RefCell, rc::Rc};

pub fn new_mapper(cartridge: Cartridge) -> (PrgMapper, ChrMapper) {
    let state = Rc::new(RefCell::new(MapperState {
        prg_mem: cartridge.prg_rom.clone(),
        chr_mem: if cartridge.chr_banks == 0 {
            [0u8; 8 * 1024].to_vec()
        } else {
            cartridge.chr_rom.clone()
        },
        cur_bank: 0,
        last_bank: cartridge.prg_banks - 1
    }));

    let prg_mapper = PrgMapper {
        state: state.clone(),
    };
    let chr_mapper = ChrMapper { state: state };

    (prg_mapper, chr_mapper)
}

struct MapperState {
    prg_mem: Vec<u8>,
    chr_mem: Vec<u8>,
    cur_bank: usize,
    last_bank: usize,
}

pub struct PrgMapper {
    state: SharedMut<MapperState>,
}

impl PrgMapper {
    fn map_address(&self, address: u16) -> u16 {
        if address <= 0x3FFF {
            self.state.borrow().cur_bank as u16 * 0x4000 + (address & 0x3FFF)
        } else {
            self.state.borrow().last_bank as u16 * 0x4000 + (address & 0x3FFF)
        }
    }
}

impl Device for PrgMapper {
    fn read(&mut self, address: u16) -> u8 {
        let address = self.map_address(address);
        self.state.borrow().prg_mem[address as usize]
    }

    fn write(&mut self, _address: u16, data: u8) {
        self.state.borrow_mut().cur_bank = (data & 0x0F) as usize;
    }
}

pub struct ChrMapper {
    state: SharedMut<MapperState>,
}

impl Device for ChrMapper {
    fn read(&mut self, address: u16) -> u8 {
        self.state.borrow().chr_mem[address as usize]
    }

    fn write(&mut self, _address: u16, _data: u8) {
        panic!("cant write to ROM");
    }
}
