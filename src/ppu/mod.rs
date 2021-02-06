use crate::bus::{Bus, BusRead, BusWrite};

pub struct PPU<'a> {
    cycles: u16,
    scanline: u16,
    render: bool,

    pub(in crate) bus: Bus<'a>,
}

impl<'a> PPU<'a> {
    pub fn new(bus: Bus<'a>) -> PPU<'a> {
        PPU {
            cycles: 0,
            scanline: 0,
            render: false,
            bus,
        }
    }

    pub fn clock(&mut self) {
        self.cycles += 1;

        if self.cycles >= 341 {
            self.cycles = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = 0; // -1?
                self.render = true;
            }
        }
    }
}

impl<'a> BusWrite for PPU<'a> {
    fn write(&mut self, _address: u16, _data: u8) {
        todo!()
    }
}

impl<'a> BusRead for PPU<'a> {
    fn read(&self, address: u16) -> u8 {
        println!("PPU STATUS read from 0x{:04X}", address);
        match address {
            0x0000 => 0x00,
            0x0001 => 0x00,
            0x0002 => 0x00,
            0x0003 => 0x00,
            0x0004 => 0x00,
            0x0005 => 0x00,
            0x0006 => 0x00,
            0x0007 => 0x00,
            _ => panic!("unknown PPU register"),
        }
    }
}
