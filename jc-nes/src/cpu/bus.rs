use crate::bus::Device;
use crate::ppu::Ppu;
use crate::ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Bus {
    ram: Ram,
    ppu_regs: Rc<RefCell<Ppu>>,

    pub prg_mapper: Option<Box<dyn Device>>,
    pub chr_mapper: Option<Box<dyn Device>>,
}

impl Bus {
    pub fn new(ppu_regs: Rc<RefCell<Ppu>>) -> Self {
        Bus {
            ram: Ram::new(vec![0u8; 2 * 1024]),
            ppu_regs,
            prg_mapper: None,
            chr_mapper: None,
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            // CPU RAM and mirror
            0x0000..0x0800 => self.ram.read(address),
            0x0800..0x2000 => self.read(address % 0x0800),

            // PPU and mirror
            0x2000..0x2008 => self.ppu_regs.borrow_mut().read(address - 0x2000),
            0x2008..0x4000 => self.read(address % 0x8),

            0x8000..=0xFFFF => self
                .prg_mapper
                .as_mut()
                .map(|mapper| mapper.read(address - 0x8000))
                .unwrap(),

            _ => panic!("out of bounds 0x{:08x}", address),
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            // CPU RAM and mirror
            0x0000..0x0800 => self.ram.write(address, data),
            0x0800..0x2000 => self.write(address % 0x0800, data),

            // PPU and mirror
            0x2000..0x2008 => self.ppu_regs.borrow_mut().write(address - 0x2000, data),

            0x2008..0x4000 => self.write(address % 0x8, data),

            0x8000..=0xFFFF => todo!("PROGRAM MAPPER"),

            _ => panic!("out of bounds 0x{:08x}", address),
        };
    }
}
