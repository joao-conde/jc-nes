use crate::bus::{Bus, BusRead, BusWrite};

pub struct PPU<'a> {
    pub(in crate) bus: Bus<'a>,
}

impl<'a> PPU<'a> {
    pub fn new(bus: Bus<'a>) -> PPU<'a> {
        PPU { bus }
    }

    pub fn clock(&mut self) {}
}

impl<'a> BusWrite for PPU<'a> {
    fn write(&mut self, _address: u16, _data: u8) {
        todo!()
    }
}

impl<'a> BusRead for PPU<'a> {
    fn read(&self, address: u16) -> u8 {
        todo!()
    }
}
