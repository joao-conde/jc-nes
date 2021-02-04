use crate::bus::BusWrite;

pub struct PPU {
    name_tables: [u8; 8 * 1024],
    palette: [u8; 255],
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            name_tables: [0u8; 8 * 1024],
            palette: [0u8; 255],
        }
    }

    pub fn clock(&self) {
        println!("clock PPU");
    }
}

impl BusWrite for PPU {
    fn write(&mut self, address: u16, data: u8) {
        todo!()
        // address & 0x0007
    }
}
