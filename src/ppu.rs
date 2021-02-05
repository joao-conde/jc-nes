use crate::bus::BusWrite;

pub struct PPU {
    _name_tables: [u8; 8 * 1024],
    _palette: [u8; 255],
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            _name_tables: [0u8; 8 * 1024],
            _palette: [0u8; 255],
        }
    }

    pub fn clock(&self) {
        println!("clock PPU");
    }
}

impl BusWrite for PPU {
    fn write(&mut self, _address: u16, _data: u8) {
        todo!()
        // address & 0x0007
    }
}
