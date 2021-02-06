use crate::bus::{Bus, BusRead, BusWrite};

pub struct PPU<'a> {
    cycles: u16,
    scanline: u16,
    render: bool,

    status: u8,
    mask: u8,
    control: u8,

    pub(in crate) bus: Bus<'a>,
}

pub(in crate::ppu) enum BitRegister {
    Status,
    Mask,
    Control,
}

pub(in crate::ppu) enum Status {
    SpriteOverflow = 0,
    SpriteZeroHit = 1,
    VerticalBlank = 2,
}

pub(in crate::ppu) enum Mask {
    GrayScale = 0,
    RenderBackGroundLeft = 1,
    RenderSpritesLeft = 2,
    RenderBackGround = 3,
    RenderSprites = 4,
    EnhanceRed = 5,
    EnhanceGreen = 6,
    EnhanceBlue = 7,
}

pub(in crate::ppu) enum Control {
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
            cycles: 0,
            scanline: 0,
            render: false,
            status: 0x00,
            mask: 0x00,
            control: 0x00,
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
