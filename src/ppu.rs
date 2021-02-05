use crate::bus::{Bus, BusWrite};

pub struct PPU<'a> {
    _name_tables: [u8; 8 * 1024],
    _palette: [u8; 255],
    pub(in crate) bus: Bus<'a>,
}

impl<'a> PPU<'a> {
    pub fn new(bus: Bus<'a>) -> PPU<'a> {
        PPU {
            _name_tables: [0u8; 8 * 1024],
            _palette: [0u8; 255],
            bus,
        }
    }

    pub fn clock(&mut self) {
        println!("clock PPU");
    }
}

impl<'a> BusWrite for PPU<'a> {
    fn write(&mut self, _address: u16, _data: u8) {
        todo!()
        // address & 0x0007
    }
}
