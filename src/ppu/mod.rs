pub mod nametable;
pub mod palette;

use crate::bus::{Bus, BusRead, BusWrite};
use bitflags::bitflags;

/// https://wiki.nesdev.com/w/index.php/PPU_scrolling#Explanation

pub struct PPU<'a> {
    cycle: u16,
    scanline: u16,
    render: bool,

    status: Status,
    mask: Mask,
    control: Control,

    write_flip_flop: bool,
    buffer: u8,

    // loopy registers (nesdev.com/loopyppu.zip)
    vram_address: VRAMAddress,
    tram_address: VRAMAddress,
    tile_offset_x: u8,
    
    pub(in crate) raise_nmi: bool,
    pub(in crate) bus: Bus<'a>,
}

bitflags! {
    struct Status: u8 {
        const SPRITE_OVERFLOW = 0x20;
        const SPRITE_ZERO_HIT = 0x40;
        const VERTICAL_BLANK = 0x80;
    }
}

bitflags! {
    struct Mask: u8 {
        const GRAY_SCALE = 0x01;
        const RENDER_BACKGROUND_LEFT = 0x02;
        const RENDER_SPRITES_LEFT = 0x04;
        const RENDER_BACKGROUND = 0x08;
        const RENDER_SPRITES = 0x10;
        const ENHANCE_RED = 0x20;
        const ENHANCE_GREEN = 0x40;
        const ENHANCE_BLUE = 0x80;
    }
}

bitflags! {
    struct Control: u8 {
        const NAMETABLE_X = 0x01;
        const NAMETABLE_Y = 0x02;
        const INCREMENT_MODE = 0x04;
        const PATTERN_SPRITE = 0x08;
        const PATTERN_BACKGROUND = 0x10;
        const SPRITE_SIZE = 0x20;
        const SLAVE_MODE = 0x40;
        const ENABLE_NMI = 0x80;
    }
}

bitflags! {
    struct VRAMAddress: u16 {
        const COARSE_X = 0x001F;
        const COARSE_Y = 0x03E0;
        const NAMETABLE_X = 0x0400;
        const NAMETABLE_Y = 0x0800;
        const FINE_Y = 0x7000;
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
            write_flip_flop: true,
            buffer: 0x00,
            raise_nmi: false,
            vram_address: VRAMAddress::from_bits_truncate(0x0000),
            tram_address: VRAMAddress::from_bits_truncate(0x0000),
            tile_offset_x: 0,
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
                if self.write_flip_flop {
                    self.tram_address = VRAMAddress::from_bits_truncate((self.tram_address.bits() & 0x00FF) | (data as u16) << 8);
                    self.write_flip_flop = false;
                } else {
                    self.tram_address = VRAMAddress::from_bits_truncate((self.tram_address.bits() & 0xFF00) | data as u16);
                    self.vram_address = self.tram_address;
                    self.write_flip_flop = true;
                }
            }
            0x0007 => {
                self.bus.write(self.vram_address.bits(), data);
                self.vram_address = VRAMAddress::from_bits_truncate(self.vram_address.bits() + 1);
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
                self.write_flip_flop = true;
                data
            }
            0x0003 => 0x00,
            0x0004 => 0x00,
            0x0005 => 0x00,
            0x0006 => 0x00,
            0x0007 => {
                let data = self.buffer;
                self.buffer = self.bus.read(self.vram_address.bits());
                // TODO immediate for palette
                let increment =  if self.control.contains(Control::INCREMENT_MODE) {32} else {1};
                self.vram_address = VRAMAddress::from_bits_truncate(self.vram_address.bits() + increment);
                data
            }
            _ => panic!("unknown PPU register"),
        }
    }
}
