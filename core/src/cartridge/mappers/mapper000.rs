use crate::bus::{Device, SharedMut};
use crate::cartridge::Cartridge;
use std::{cell::RefCell, rc::Rc};

pub fn new_mapper(cartridge: Cartridge) -> (PrgMapper, ChrMapper) {
    let state = Rc::new(RefCell::new(MapperState {
        prg_mem: cartridge.prg_rom.clone(),
        prg_banks: cartridge.prg_banks as usize,
        chr_mem: if cartridge.chr_banks == 0 {
            [0u8; 8 * 1024].to_vec()
        } else {
            cartridge.chr_rom.clone()
        },
    }));

    let prg_mapper = PrgMapper {
        state: state.clone(),
    };
    let chr_mapper = ChrMapper { state: state };

    (prg_mapper, chr_mapper)
}

struct MapperState {
    prg_mem: Vec<u8>,
    prg_banks: usize,
    chr_mem: Vec<u8>,
}

pub struct PrgMapper {
    state: SharedMut<MapperState>,
}

impl PrgMapper {
    fn map_address(&self, address: u16) -> u16 {
        if self.state.borrow().prg_banks == 1 {
            address & 0x3FFF
        } else {
            address & 0x7FFF
        }
    }
}

impl Device for PrgMapper {
    fn read(&mut self, address: u16) -> u8 {
        let address = self.map_address(address);
        self.state.borrow().prg_mem[address as usize]
    }

    fn write(&mut self, _address: u16, _data: u8) {
        panic!("cant write to ROM");
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
