mod vram_address;

use crate::bus::{Bus, Device};
use crate::ppu::vram_address::VRAMAddress;
use bitflags::bitflags;

pub struct PPU<'a> {
    cycle: u16,
    scanline: i16,

    pub render: bool,

    status: Status,
    mask: Mask,
    control: Control,

    write_flip_flop: bool,
    buffer: u8,

    pub screen: [[(u8, u8, u8); 256]; 240],

    dac: [(u8, u8, u8); 0x40],

    // address registers (nesdev.com/loopyppu.zip)
    // https://wiki.nesdev.com/w/index.php/PPU_scrolling#Explanation
    vram_address: VRAMAddress,
    tram_address: VRAMAddress,
    fine_x: u8,

    // background buffered data (pre-load)
    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,

    // background shifters
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,

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

impl<'a> PPU<'a> {
    pub fn new(bus: Bus<'a>) -> PPU<'a> {
        let dac = [
            (84, 84, 84),
            (0, 30, 116),
            (8, 16, 144),
            (48, 0, 136),
            (68, 0, 100),
            (92, 0, 48),
            (84, 4, 0),
            (60, 24, 0),
            (32, 42, 0),
            (8, 58, 0),
            (0, 64, 0),
            (0, 60, 0),
            (0, 50, 60),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (152, 150, 152),
            (8, 76, 196),
            (48, 50, 236),
            (92, 30, 228),
            (136, 20, 176),
            (160, 20, 100),
            (152, 34, 32),
            (120, 60, 0),
            (84, 90, 0),
            (40, 114, 0),
            (8, 124, 0),
            (0, 118, 40),
            (0, 102, 120),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (236, 238, 236),
            (76, 154, 236),
            (120, 124, 236),
            (176, 98, 236),
            (228, 84, 236),
            (236, 88, 180),
            (236, 106, 100),
            (212, 136, 32),
            (160, 170, 0),
            (116, 196, 0),
            (76, 208, 32),
            (56, 204, 108),
            (56, 180, 204),
            (60, 60, 60),
            (0, 0, 0),
            (0, 0, 0),
            (236, 238, 236),
            (168, 204, 236),
            (188, 188, 236),
            (212, 178, 236),
            (236, 174, 236),
            (236, 174, 212),
            (236, 180, 176),
            (228, 196, 144),
            (204, 210, 120),
            (180, 222, 120),
            (168, 226, 144),
            (152, 226, 180),
            (160, 214, 228),
            (160, 162, 160),
            (0, 0, 0),
            (0, 0, 0),
        ];

        PPU {
            cycle: 0,
            scanline: 0,
            render: false,
            status: Status::from_bits_truncate(0x00),
            mask: Mask::from_bits_truncate(0x00),
            control: Control::from_bits_truncate(0x00),
            write_flip_flop: true,
            buffer: 0x00,
            screen: [[(0, 0, 0); 256]; 240],
            raise_nmi: false,
            vram_address: VRAMAddress::default(),
            tram_address: VRAMAddress::default(),
            fine_x: 0x00,
            bg_next_tile_id: 0x00,
            bg_next_tile_attrib: 0x00,
            bg_next_tile_lsb: 0x00,
            bg_next_tile_msb: 0x00,
            bg_shifter_pattern_lo: 0x0000,
            bg_shifter_pattern_hi: 0x0000,
            bg_shifter_attrib_lo: 0x0000,
            bg_shifter_attrib_hi: 0x0000,
            dac,
            bus,
        }
    }

    // https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
    pub fn clock(&mut self) {
        // visible frame
        if self.scanline >= -1 && self.scanline < 240 {
            // odd frame skip
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }

            // new frame
            if self.scanline == -1 && self.cycle == 1 {
                self.status.set(Status::VERTICAL_BLANK, false);
                self.render = false;
            }

            if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
                self.shift_background_shifters();

                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_background_shifters();
                        self.bg_next_tile_id = self
                            .bus
                            .read(0x2000 | (u16::from(self.vram_address) & 0x0FFF));
                    }
                    2 => {
                        self.bg_next_tile_attrib = self.bus.read(
                            0x23C0
                                | ((self.vram_address.nametable_y as u16) << 11)
                                | ((self.vram_address.nametable_x as u16) << 10)
                                | (((self.vram_address.coarse_y as u16) >> 2) << 3)
                                | (self.vram_address.coarse_x as u16) >> 2,
                        );

                        if (self.vram_address.coarse_y & 0x0002) != 0 {
                            self.bg_next_tile_attrib >>= 4;
                        }

                        if (self.vram_address.coarse_x & 0x0002) != 0 {
                            self.bg_next_tile_attrib >>= 2;
                        }

                        self.bg_next_tile_attrib &= 0x03;
                    }
                    4 => {
                        self.bg_next_tile_lsb = self.bus.read(
                            (((self.control & Control::PATTERN_BACKGROUND).bits() as u16) << 12)
                                + ((self.bg_next_tile_id as u16) << 4)
                                + self.vram_address.fine_y as u16,
                        );
                    }
                    6 => {
                        self.bg_next_tile_msb = self.bus.read(
                            (((self.control & Control::PATTERN_BACKGROUND).bits() as u16) << 12)
                                + ((self.bg_next_tile_id as u16) << 4)
                                + self.vram_address.fine_y as u16
                                + 8,
                        );
                    }
                    7 => self.inc_x(),
                    _ => (),
                }
            }

            if self.cycle == 256 {
                self.inc_y();
            }

            if self.cycle == 257 {
                self.load_background_shifters();
                self.reset_x();
            }

            if self.cycle == 338 || self.cycle == 340 {
                self.bg_next_tile_id = self
                    .bus
                    .read(0x2000 | (u16::from(self.vram_address) & 0x0FFF));
            }

            if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
                self.reset_y();
            }
        }
        // post-render
        else if self.scanline >= 241 && self.scanline < 261 {
            if self.scanline == 241 && self.cycle == 1 {
                self.status.set(Status::VERTICAL_BLANK, true);
                self.render = true;
                if self.control.contains(Control::ENABLE_NMI) {
                    self.raise_nmi = true;
                }
            }
        }

        let color = if self.mask.contains(Mask::RENDER_BACKGROUND) {
            let bit_offset = 0x8000 >> self.fine_x;

            let p0_pixel = (self.bg_shifter_pattern_lo & bit_offset) > 0;
            let p1_pixel = (self.bg_shifter_pattern_hi & bit_offset) > 0;
            let bg_pixel = ((p1_pixel as u8) << 1) | p0_pixel as u8;

            // palette
            let bg_pal0 = ((self.bg_shifter_attrib_lo & bit_offset) > 0) as u8;
            let bg_pal1 = ((self.bg_shifter_attrib_hi & bit_offset) > 0) as u8;
            let bg_palette = (bg_pal1 << 1) | bg_pal0;

            let color_i = self
                .bus
                .read(0x3F00 + (bg_palette << 2) as u16 + bg_pixel as u16);
            self.dac[color_i as usize]
        } else {
            (0, 0, 0)
        };

        if self.cycle > 0 && self.cycle < 256 && self.scanline >= 0 && self.scanline < 240 {
            self.screen[self.scanline as usize][self.cycle as usize - 1] = color;
        }

        self.cycle += 1;

        // reset cycle
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
        }

        // reset scanlines
        if self.scanline >= 261 {
            self.scanline = -1;
            self.render = true;
        }
    }

    pub fn reset(&mut self) {
        self.fine_x = 0x00;
        self.write_flip_flop = true;
        self.buffer = 0x00;
        self.scanline = 0;
        self.cycle = 0;
        self.bg_next_tile_id = 0x00;
        self.bg_next_tile_attrib = 0x00;
        self.bg_next_tile_lsb = 0x00;
        self.bg_next_tile_msb = 0x00;
        self.bg_shifter_pattern_lo = 0x0000;
        self.bg_shifter_pattern_hi = 0x0000;
        self.bg_shifter_attrib_lo = 0x0000;
        self.bg_shifter_attrib_hi = 0x0000;
        self.status = Status::from_bits_truncate(0x00);
        self.mask = Mask::from_bits_truncate(0x00);
        self.control = Control::from_bits_truncate(0x00);
        self.vram_address = VRAMAddress::default();
        self.tram_address = VRAMAddress::default();
    }

    pub fn debug(&self) {
        println!(
            " PPU {}, {} VRAM: 0x{:4X}",
            self.scanline,
            self.cycle,
            u16::from(self.vram_address)
        );
    }

    pub fn pause(&self) {
        use std::io::stdin;
        let mut s = String::new();
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
    }
}

impl<'a> PPU<'a> {
    fn inc_x(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND) || self.mask.contains(Mask::RENDER_SPRITES) {
            self.vram_address.coarse_x += 1;
            if self.vram_address.coarse_x == 32 {
                self.vram_address.coarse_x = 0;
                self.vram_address.nametable_x = if self.vram_address.nametable_x == 0 {
                    1
                } else {
                    0
                };
            }
        }
    }

    fn inc_y(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND) || self.mask.contains(Mask::RENDER_SPRITES) {
            if self.vram_address.fine_y < 7 {
                self.vram_address.fine_y += 1;
            } else {
                self.vram_address.fine_y = 0;
                if self.vram_address.coarse_y == 29 {
                    self.vram_address.coarse_y = 0;
                    self.vram_address.nametable_y = if self.vram_address.nametable_y == 0 {
                        1
                    } else {
                        0
                    };
                } else if self.vram_address.coarse_y == 31 {
                    self.vram_address.coarse_y = 0;
                } else {
                    self.vram_address.coarse_y += 1;
                }
            }
        }
    }

    fn reset_x(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND) || self.mask.contains(Mask::RENDER_SPRITES) {
            self.vram_address.nametable_x = self.tram_address.nametable_x;
            self.vram_address.coarse_x = self.tram_address.coarse_x;
        }
    }

    fn reset_y(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND) || self.mask.contains(Mask::RENDER_SPRITES) {
            self.vram_address.nametable_y = self.tram_address.nametable_y;
            self.vram_address.coarse_y = self.tram_address.coarse_y;
            self.vram_address.fine_y = self.tram_address.fine_y;
        }
    }

    fn shift_background_shifters(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND) {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }
    }

    fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo =
            (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_shifter_pattern_hi =
            (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;
        self.bg_shifter_attrib_lo =
            (self.bg_shifter_attrib_lo & 0xFF00) | ((self.bg_next_tile_attrib & 0b01) as u16);
        self.bg_shifter_attrib_hi =
            (self.bg_shifter_attrib_hi & 0xFF00) | ((self.bg_next_tile_attrib & 0b10) as u16);
    }
}

impl<'a> Device for PPU<'a> {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000 => 0x00,
            0x0001 => 0x00,
            0x0002 => {
                let data = self.status.bits() & 0xE0 | (self.buffer & 0x1F);
                self.status.set(Status::VERTICAL_BLANK, false);
                self.render = false;
                self.write_flip_flop = true;
                data
            }
            0x0003 => 0x00,
            0x0004 => 0x00,
            0x0005 => 0x00,
            0x0006 => 0x00,
            0x0007 => {
                let mut data = self.buffer;
                self.buffer = self.bus.read(self.vram_address.into());

                if u16::from(self.vram_address) >= 0x3F00 {
                    data = self.buffer
                };

                let increment = if self.control.contains(Control::INCREMENT_MODE) {
                    32
                } else {
                    1
                } as u16;
                self.vram_address = (u16::from(self.vram_address) + increment).into();
                data
            }
            _ => panic!("unknown PPU register"),
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000 => {
                self.control = Control::from_bits_truncate(data);
                self.tram_address.nametable_x = (self.control & Control::NAMETABLE_X).bits();
                self.tram_address.nametable_y = (self.control & Control::NAMETABLE_Y).bits();
            }
            0x0001 => {
                self.mask = Mask::from_bits_truncate(data);
            }
            0x0002 => (),
            0x0003 => (),
            0x0004 => (),
            0x0005 => {
                if self.write_flip_flop {
                    self.fine_x = data & 0x07;
                    self.tram_address.coarse_x = data >> 3;
                    self.write_flip_flop = false;
                } else {
                    self.tram_address.fine_y = data & 0x07;
                    self.tram_address.coarse_y = data >> 3;
                    self.write_flip_flop = true;
                }
            }
            0x0006 => {
                if self.write_flip_flop {
                    let addr: u16 =
                        (u16::from(self.tram_address) & 0x00FF) | (u16::from(data) << 8);
                    self.tram_address = addr.into();

                    // println!("addr: 0x{:4X}", addr);
                    // println!("data: 0x{:2X}", data);
                    // println!("set tram to 0x{:4X}", u16::from(self.tram_address));
                    self.write_flip_flop = false;
                } else {
                    let addr: u16 = (u16::from(self.tram_address) & 0xFF00) | u16::from(data);
                    self.tram_address = addr.into();
                    self.vram_address = self.tram_address;

                    // println!("addr: 0x{:4X}", addr);
                    // println!("data: 0x{:2X}", data);
                    // println!("set tram to 0x{:4X}", u16::from(self.tram_address));
                    // println!("set vram to 0x{:4X}", u16::from(self.vram_address));
                    self.write_flip_flop = true;
                }
            }
            0x0007 => {
                // self.debug();
                // println!("v: {:?}", self.vram_address);
                // println!("t: {:?}", self.tram_address);

                self.bus.write(0x2000 | u16::from(self.vram_address), data);
                let increment = if self.control.contains(Control::INCREMENT_MODE) {
                    32
                } else {
                    1
                } as u16;
                self.vram_address = (u16::from(self.vram_address) + increment).into();
            }
            _ => panic!("unknown PPU register"),
        }
    }
}
