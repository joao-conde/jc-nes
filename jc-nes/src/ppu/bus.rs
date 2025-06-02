use super::palette::Palette;
use crate::bus::Device;
use crate::ram::Ram;

pub struct Bus {
    patterntbl1: Ram,
    patterntbl2: Ram,
    nametbl1: Ram,
    nametbl2: Ram,
    nametbl3: Ram,
    nametbl4: Ram,
    palette: Palette,
    pub chr_mapper: Option<Box<dyn Device>>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            patterntbl1: Ram::new(vec![0u8; 4 * 1024]),
            patterntbl2: Ram::new(vec![0u8; 4 * 1024]),
            nametbl1: Ram::new(vec![0u8; 1024]),
            nametbl2: Ram::new(vec![0u8; 1024]),
            nametbl3: Ram::new(vec![0u8; 1024]),
            nametbl4: Ram::new(vec![0u8; 1024]),
            palette: Palette::new(),
            chr_mapper: None,
        }
    }

    pub fn connect_chr_mapper(&mut self, mapper: impl Device + 'static) {
        self.chr_mapper = Some(Box::new(mapper));
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..0x1000 => {
                if let Some(ref mut mapper) = self.chr_mapper {
                    mapper.read(address)
                } else {
                    self.patterntbl1.read(address)
                }
            }
            0x1000..0x2000 => {
                if let Some(ref mut mapper) = self.chr_mapper {
                    mapper.read(address)
                } else {
                    self.patterntbl2.read(address)
                }
            }
            0x2000..0x2400 => self.nametbl1.read(address - 0x2000),
            0x2400..0x2800 => self.nametbl2.read(address - 0x2400),
            0x2800..0x2C00 => self.nametbl3.read(address - 0x2800),
            0x2C00..0x3000 => self.nametbl4.read(address - 0x2C00),
            0x3000..0x3F00 => self.read(address - 0x3000),
            0x3F00..0x3F20 => self.palette.read(address - 0x3F00),
            0x3F20..0x4000 => self.palette.read(address % 0x20),
            _ => panic!("out of bounds ppu read 0x{:04x}", address),
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..0x1000 => {
                if let Some(ref mut mapper) = self.chr_mapper {
                    mapper.write(address, data)
                } else {
                    self.patterntbl1.write(address, data)
                }
            }
            0x1000..0x2000 => {
                if let Some(ref mut mapper) = self.chr_mapper {
                    mapper.write(address, data)
                } else {
                    self.patterntbl2.write(address, data)
                }
            }
            0x2000..0x2400 => self.nametbl1.write(address, data),
            0x2400..0x2800 => self.nametbl2.write(address, data),
            0x2800..0x2C00 => self.nametbl3.write(address, data),
            0x2C00..0x3000 => self.nametbl4.write(address, data),
            0x3000..0x3F00 => self.write(address, data),
            0x3F00..0x3F20 => self.palette.write(address, data),
            0x3F20..0x4000 => self.palette.write(address % 0x20, data),
            _ => panic!("out of bounds ppu write 0x{:04x}", address),
        };
    }
}
