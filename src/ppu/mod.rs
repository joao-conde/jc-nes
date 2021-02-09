pub mod nametable;
pub mod palette;

use crate::bus::{Bus, BusRead, BusWrite};
use bitflags::bitflags;

/// nesdev.com/loopyppu.zip
/// https://wiki.nesdev.com/w/index.php/PPU_scrolling#Summary

pub struct PPU<'a> {
    cycle: u16,
    scanline: u16,
    render: bool,

    status: Status,
    mask: Mask,
    control: Control,

    address_latch_hi: bool,
    buffer: u8,
    address: u16,

    pub(in crate) raise_nmi: bool,
    pub(in crate) bus: Bus<'a>,
}

bitflags! {
    struct Status: u8 {
        const SPRITE_OVERFLOW = 0b00100000;
        const SPRITE_ZERO_HIT = 0b01000000;
        const VERTICAL_BLANK = 0b10000000;
    }
}

bitflags! {
    struct Mask: u8 {
        const GRAY_SCALE = 0b00000001;
        const RENDER_BACKGROUND_LEFT = 0b00000010;
        const RENDER_SPRITES_LEFT = 0b00000100;
        const RENDER_BACKGROUND = 0b00001000;
        const RENDER_SPRITES = 0b00010000;
        const ENHANCE_RED = 0b00100000;
        const ENHANCE_GREEN = 0b01000000;
        const ENHANCE_BLUE = 0b10000000;
    }
}

bitflags! {
    struct Control: u8 {
        const NAMETABLE_X = 0b00000001;
        const NAMETABLE_Y = 0b00000010;
        const INCREMENT_MODE = 0b00000100;
        const PATTERN_SPRITE = 0b00001000;
        const PATTERN_BACKGROUND = 0b00010000;
        const SPRITE_SIZE = 0b00100000;
        const SLAVE_MODE = 0b01000000;
        const ENABLE_NMI = 0b10000000;
    }
}

impl<'a> PPU<'a> {
    pub fn new(bus: Bus<'a>) -> PPU<'a> {
        PPU {
            cycle: 0,
            scanline: 0,
            render: false,
            status: Status::from_bits_truncate(0x00),
            mask: Mask::from_bits_truncate(0x00),
            control: Control::from_bits_truncate(0x00),
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
            self.status.set(Status::VERTICAL_BLANK, false);
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status.set(Status::VERTICAL_BLANK, true);

            if (self.control & Control::ENABLE_NMI).bits() != 0 {
                self.raise_nmi = true
            }
        }

        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = 0;
                self.render = true;
            }
        }
    }
}

impl<'a> BusWrite for PPU<'a> {
    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000 => self.control = Control::from_bits_truncate(data),
            0x0001 => self.mask = Mask::from_bits_truncate(data),
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
                self.status.set(Status::VERTICAL_BLANK, true); //hack
                let data = self.status.bits() & 0xE0 | (self.buffer & 0x1F);
                self.status.set(Status::VERTICAL_BLANK, false);
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
