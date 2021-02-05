use crate::bus::{BusRead, BusWrite};
use crate::cartridge::Cartridge;
use crate::mappers::MapperMemoryPin;
use std::cell::RefCell;
use std::rc::Rc;

// DK SPECIFIC
pub struct Mapper000 {
    pin: MapperMemoryPin,
    cartridge: Rc<RefCell<Cartridge>>,
    prg_banks: u8,
}

impl Mapper000 {
    pub fn new(
        pin: MapperMemoryPin,
        cartridge: &Rc<RefCell<Cartridge>>,
        prg_banks: u8,
    ) -> Mapper000 {
        Mapper000 {
            pin,
            cartridge: Rc::clone(cartridge),
            prg_banks,
        }
    }

    fn read_cpu(&self, address: u16) -> u8 {
        let address = if address >= 0x8000 && address < 0xFFFF {
            address & if self.prg_banks > 1 { 0x7FFF } else { 0x3FFF }
        } else {
            address
        };

        (*self.cartridge).borrow().read_prg_rom(address)
    }

    fn read_ppu(&self, address: u16) -> u8 {
        (*self.cartridge).borrow().read_chr_rom(address)
    }

    fn write_cpu(&mut self, address: u16, data: u8) {
        let address = if address >= 0x8000 && address < 0xFFFF {
            address & if self.prg_banks > 1 { 0x7FFF } else { 0x3FFF }
        } else {
            address
        };

        (*self.cartridge).borrow_mut().write_prg_rom(address, data);
    }
}

impl BusRead for Mapper000 {
    fn read(&self, address: u16) -> u8 {
        match self.pin {
            MapperMemoryPin::PrgROM => self.read_cpu(address),
            MapperMemoryPin::ChrROM => self.read_ppu(address),
        }
    }
}

impl BusWrite for Mapper000 {
    fn write(&mut self, address: u16, data: u8) {
        match self.pin {
            MapperMemoryPin::PrgROM => self.write_cpu(address, data),
            MapperMemoryPin::ChrROM => (),
        };
    }
}
