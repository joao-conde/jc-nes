use crate::cartridge::mappers::MapperMemoryPin;
use crate::cartridge::Cartridge;
use crate::{
    bus::{BusRead, BusWrite},
    nes::SharedMut,
};
use std::rc::Rc;

// DK SPECIFIC
pub struct Mapper000 {
    pin: MapperMemoryPin,
    cartridge: SharedMut<Cartridge>,
    prg_banks: u8,
}

impl BusRead for Mapper000 {
    fn read(&self, address: u16) -> u8 {
        match self.pin {
            MapperMemoryPin::PrgROM => self.read_prg_rom(address),
            MapperMemoryPin::ChrROM => self.read_chr_rom(address),
        }
    }
}

impl BusWrite for Mapper000 {
    fn write(&mut self, address: u16, data: u8) {
        match self.pin {
            MapperMemoryPin::PrgROM => self.write_prg_rom(address, data),
            MapperMemoryPin::ChrROM => self.write_chr_rom(address, data),
        };
    }
}

impl Mapper000 {
    pub fn new(pin: MapperMemoryPin, cartridge: &SharedMut<Cartridge>) -> Mapper000 {
        Mapper000 {
            pin,
            prg_banks: cartridge.borrow().meta.prg_banks,
            cartridge: Rc::clone(cartridge),
        }
    }

    fn read_prg_rom(&self, address: u16) -> u8 {
        let address = if self.prg_banks == 1 {
            address & 0x3FFF
        } else {
            address
        };

        (*self.cartridge).borrow().prg_rom[address as usize]
    }

    fn read_chr_rom(&self, address: u16) -> u8 {
        (*self.cartridge).borrow().chr_rom[address as usize]
    }

    fn write_prg_rom(&mut self, address: u16, data: u8) {
        let address = if self.prg_banks == 1 {
            address & 0x3FFF
        } else {
            address
        };

        (*self.cartridge).borrow_mut().prg_rom[address as usize] = data;
    }

    fn write_chr_rom(&mut self, address: u16, data: u8) {
        (*self.cartridge).borrow_mut().chr_rom[address as usize] = data;
    }
}
