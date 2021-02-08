pub mod nametable;
pub mod palette;

use crate::bus::{Bus, BusRead, BusWrite};

pub struct PPU<'a> {
    cycle: u16,
    scanline: u16,
    render: bool,

    status: u8,
    mask: u8,
    control: u8,

    address_latch_hi: bool,
    buffer: u8,
    address: u16,

    pub(in crate) raise_nmi: bool,
    pub(in crate) bus: Bus<'a>,
}

enum BitRegister {
    Status,
    Mask,
    Control,
}

enum Status {
    SpriteOverflow = 5,
    SpriteZeroHit = 6,
    VerticalBlank = 7,
}

enum Mask {
    GrayScale = 0,
    RenderBackGroundLeft = 1,
    RenderSpritesLeft = 2,
    RenderBackGround = 3,
    RenderSprites = 4,
    EnhanceRed = 5,
    EnhanceGreen = 6,
    EnhanceBlue = 7,
}

enum Control {
    NameTableX = 0,
    NameTableY = 1,
    IncrementMode = 2,
    PatternSprite = 3,
    PatternBackground = 4,
    SpriteSize = 5,
    SlaveMode = 6,
    EnableNMI = 7,
}

impl<'a> PPU<'a> {
    pub fn new(bus: Bus<'a>) -> PPU<'a> {
        PPU {
            cycle: 0,
            scanline: 0,
            render: false,
            status: 0x00,
            mask: 0x00,
            control: 0x00,
            address_latch_hi: true,
            address: 0x0000,
            buffer: 0x00,
            raise_nmi: false,
            bus,
        }
    }

    // https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
    pub fn clock(&mut self) {
        if self.scanline == 0 && self.cycle == 1 {
            self.set_register_bit(BitRegister::Status, Status::VerticalBlank as u8, false);
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.set_register_bit(BitRegister::Status, Status::VerticalBlank as u8, true);

            if self.is_set(BitRegister::Control, Control::EnableNMI as u8) {
                self.raise_nmi = true
            }
        }

        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = 0; // -1?
                self.render = true;
            }
        }
    }
}

impl<'a> PPU<'a> {
    fn set_register_bit(&mut self, reg: BitRegister, bit: u8, set_condition: bool) {
        if set_condition {
            match reg {
                BitRegister::Status => self.status |= 1 << bit as u8,
                BitRegister::Mask => self.mask |= 1 << bit as u8,
                BitRegister::Control => self.control |= 1 << bit as u8,
            }
        } else {
            match reg {
                BitRegister::Status => self.status &= !(1 << bit as u8),
                BitRegister::Mask => self.mask &= !(1 << bit as u8),
                BitRegister::Control => self.control &= !(1 << bit as u8),
            }
        }
    }

    fn is_set(&self, reg: BitRegister, bit: u8) -> bool {
        match reg {
            BitRegister::Status => (self.status >> bit as u8) & 1 == 1,
            BitRegister::Mask => (self.mask >> bit as u8) & 1 == 1,
            BitRegister::Control => (self.control >> bit as u8) & 1 == 1,
        }
    }
}

impl<'a> BusWrite for PPU<'a> {
    fn write(&mut self, address: u16, data: u8) {
        // println!("PPU STATUS write to 0x{:04X}", address);
        match address {
            0x0000 => self.control = data,
            0x0001 => self.mask = data,
            0x0002 => (),
            0x0003 => (),
            0x0004 => (),
            0x0005 => (),
            0x0006 => {
                if self.address_latch_hi {
                    self.address = (self.address & 0x00FF) | (data as u16) << 8;
                    self.address_latch_hi = false;
                } else {
                    self.address = (self.address & 0xFF00) | data as u16;
                    self.address_latch_hi = true;
                }
            }
            0x0007 => {
                self.bus.write(self.address, data);
                self.address += 1;
            }
            _ => panic!("unknown PPU register"),
        }
    }
}

impl<'a> BusRead for PPU<'a> {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000 => 0x00,
            0x0001 => 0x00,
            0x0002 => {
                self.set_register_bit(BitRegister::Status, Status::VerticalBlank as u8, true); //hack
                let data = self.status & 0xE0 | (self.buffer & 0x1F);
                self.set_register_bit(BitRegister::Status, Status::VerticalBlank as u8, false);
                self.address_latch_hi = true;
                data
            }
            0x0003 => 0x00,
            0x0004 => 0x00,
            0x0005 => 0x00,
            0x0006 => 0x00,
            0x0007 => {
                let data = self.buffer;
                self.buffer = self.bus.read(self.address);
                // TODO immediate for palette
                self.address += 1;
                data
            }
            _ => panic!("unknown PPU register"),
        }
    }
}
