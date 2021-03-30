pub mod dma;
pub mod palette;

mod control;
mod mask;
mod oam;
mod status;
mod vram_address;

use crate::ppu::control::Control;
use crate::ppu::mask::Mask;
use crate::ppu::oam::{Sprite, OAM};
use crate::ppu::palette::PALETTE;
use crate::ppu::status::Status;
use crate::ppu::vram_address::VRAMAddress;
use crate::{
    bus::{Bus, Device},
    cartridge::MirrorMode,
};

pub const WIDTH: u16 = 256;
pub const HEIGHT: u16 = 240;

pub struct PPU {
    pub(in crate) frame_complete: bool,
    pub(in crate) screen: [u8; WIDTH as usize * HEIGHT as usize * 3],
    pub(in crate) raise_nmi: bool,
    pub(in crate) bus: Bus,
    pub(in crate) oam: OAM,
    pub(in crate) cartridge_mirror_mode: MirrorMode,

    // current screen pixel
    cycle: u16,
    scanline: i16,

    // state registers
    status: Status,
    mask: Mask,
    control: Control,

    vram_address: VRAMAddress,
    tram_address: VRAMAddress,
    fine_x: u8,

    write_flip_flop: bool,

    // buffered PPU data in between clocks
    buffer: u8,

    // background buffered data in between clocks
    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,

    // background shifters
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,

    //foreground rendering
    scanline_sprites: Vec<Sprite>,
    sprite_shifter_pattern_lo: [u8; 8],
    sprite_shifter_pattern_hi: [u8; 8],
    sprite_zero_selected: bool,
}

impl PPU {
    pub fn new(bus: Bus) -> PPU {
        PPU {
            bus,
            cycle: 0,
            scanline: 0,
            frame_complete: false,
            status: Status::from(0x00),
            mask: Mask::from(0x00),
            control: Control::from(0x00),
            write_flip_flop: true,
            buffer: 0x00,
            screen: [0; WIDTH as usize * HEIGHT as usize * 3],
            raise_nmi: false,
            vram_address: VRAMAddress::from(0x0000),
            tram_address: VRAMAddress::from(0x0000),
            fine_x: 0x00,
            bg_next_tile_id: 0x00,
            bg_next_tile_attrib: 0x00,
            bg_next_tile_lsb: 0x00,
            bg_next_tile_msb: 0x00,
            bg_shifter_pattern_lo: 0x0000,
            bg_shifter_pattern_hi: 0x0000,
            bg_shifter_attrib_lo: 0x0000,
            bg_shifter_attrib_hi: 0x0000,
            cartridge_mirror_mode: MirrorMode::Horizontal,
            oam: OAM::default(),
            scanline_sprites: Vec::with_capacity(8),
            sprite_shifter_pattern_lo: [0u8; 8],
            sprite_shifter_pattern_hi: [0u8; 8],
            sprite_zero_selected: false,
        }
    }

    // https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
    pub fn clock(&mut self) {
        // visible frame
        if self.scanline >= -1 && self.scanline < HEIGHT as i16 {
            // odd frame skip
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }

            // new frame
            if self.scanline == -1 && self.cycle == 1 {
                self.status.vertical_blank = false;
                self.status.sprite_overflow = false;
                self.status.sprite_zero_hit = false;
            }

            if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
                self.shift_shifters();

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
                            ((self.control.pattern_background as u16) << 12)
                                + ((self.bg_next_tile_id as u16) << 4)
                                + self.vram_address.fine_y as u16,
                        );
                    }
                    6 => {
                        self.bg_next_tile_msb = self.bus.read(
                            ((self.control.pattern_background as u16) << 12)
                                + ((self.bg_next_tile_id as u16) << 4)
                                + self.vram_address.fine_y as u16
                                + 8,
                        );
                    }
                    7 => self.inc_x(),
                    _ => (),
                }
            }

            if self.cycle == WIDTH {
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
        else if self.scanline == 241 && self.cycle == 1 {
            self.status.vertical_blank = true;
            if self.control.enable_nmi {
                self.raise_nmi = true;
            }
        }

        // background pixel
        let (bg_pixel, bg_palette) = if self.mask.render_background {
            let bit_offset = 0x8000 >> self.fine_x;
            let pixel_lo = ((self.bg_shifter_pattern_lo & bit_offset) != 0) as u8;
            let pixel_hi = ((self.bg_shifter_pattern_hi & bit_offset) != 0) as u8;
            let palette_lo = ((self.bg_shifter_attrib_lo & bit_offset) != 0) as u8;
            let palette_hi = ((self.bg_shifter_attrib_hi & bit_offset) != 0) as u8;
            ((pixel_hi << 1) | pixel_lo, (palette_hi << 1) | palette_lo)
        } else {
            (0x00, 0x00)
        };

        // foreground pixel
        let mut fg_pixel = 0x00;
        let mut fg_palette = 0x00;
        let mut fg_priority = false;
        let mut sprite_zero_visible = false;
        if self.mask.render_sprites {
            for (i, sprite) in self.scanline_sprites.iter().enumerate() {
                if sprite.x == 0 {
                    let fg_pixel_lo = (self.sprite_shifter_pattern_lo[i] & 0x80) >> 7;
                    let fg_pixel_hi = (self.sprite_shifter_pattern_hi[i] & 0x80) >> 7;
                    fg_pixel = (fg_pixel_hi << 1) | fg_pixel_lo;

                    fg_palette = (sprite.attr & 0x03) + 0x04;
                    fg_priority = (sprite.attr & 0x20) == 0;

                    if fg_pixel != 0 {
                        if i == 0 {
                            sprite_zero_visible = true;
                        }
                        break;
                    }
                }
            }
        }

        // resolve foreground/background priority
        let (pixel, palette) = if bg_pixel == 0 && fg_pixel == 0 {
            (0x00, 0x00)
        } else if bg_pixel == 0 && fg_pixel > 0 {
            (fg_pixel, fg_palette)
        } else if bg_pixel > 0 && fg_pixel == 0 {
            (bg_pixel, bg_palette)
        } else {
            self.status.sprite_zero_hit = self.sprite_zero_selected
                && sprite_zero_visible
                && self.mask.render_background
                && self.mask.render_sprites
                && (!self.mask.render_background_left
                    && !self.mask.render_sprites_left
                    && self.cycle >= 9
                    && self.cycle < 258
                    || self.mask.render_background_left
                        && self.mask.render_sprites_left
                        && self.cycle >= 1
                        && self.cycle < 258);

            if fg_priority {
                (fg_pixel, fg_palette)
            } else {
                (bg_pixel, bg_palette)
            }
        };

        if self.cycle > 0
            && self.cycle < WIDTH
            && self.scanline >= 0
            && self.scanline < HEIGHT as i16
        {
            let addr = 0x3F00 + ((palette as u16) << 2) + pixel as u16;
            let color_i = self.bus.read(addr);

            let tex_addr =
                WIDTH as usize * 3 * (self.scanline as usize) + (self.cycle as usize - 1) * 3;

            if self.scanline == 130 && self.cycle == 150 && color_i == 41 {
                // println!("pix: 0x{:02X} pal: 0x{:02X} addr: 0x{:04X}", pixel, palette, addr);
                // println!("bgpix: 0x{:02X} fgpix: 0x{:02X}", bg_pixel, fg_pixel);
            }

            self.screen[tex_addr as usize] = PALETTE[color_i as usize].0;
            self.screen[tex_addr as usize + 1] = PALETTE[color_i as usize].1;
            self.screen[tex_addr as usize + 2] = PALETTE[color_i as usize].2;
        }

        self.cycle += 1;

        // reset cycles
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
        }

        // reset scanlines
        if self.scanline >= 261 {
            self.scanline = -1;
            self.frame_complete = true;
        }

        // sprite evaluation phase
        if self.cycle == 257 && self.scanline >= 0 {
            self.sprite_zero_selected = false;
            self.scanline_sprites = vec![];

            let sprite_size = if self.control.sprite_size { 16 } else { 8 };
            let mut oam_i = 0;
            let mut sprite_cnt = 0;
            while oam_i < 64 && sprite_cnt < 9 {
                let sprite = Sprite::from(&self.oam.mem[oam_i * 4..oam_i * 4 + 4]);
                let diff = self.scanline - sprite.y as i16;
                if diff >= 0 && diff < sprite_size {
                    if sprite_cnt >= 8 {
                        self.status.sprite_overflow = true;
                    } else {
                        self.scanline_sprites.push(sprite);
                        if sprite_cnt == 0 {
                            // sprite-zero present
                            self.sprite_zero_selected = true;
                        }
                    }
                    sprite_cnt += 1;
                }
                oam_i += 1;
            }
        }

        // populate shifters with next scanline data
        if self.cycle == 340 {
            for (i, sprite) in self.scanline_sprites.iter().enumerate() {
                let sprite_pattern_addr_lo = if self.control.sprite_size {
                    // 8x16 mode
                    if sprite.attr & 0x80 == 0 {
                        // sprite normal
                        if self.scanline - (sprite.y as i16) < 8 {
                            // top half
                            ((sprite.tile_id as u16 & 0x01) << 12)
                                | ((sprite.tile_id & 0xFE) << 4) as u16
                                | (self.scanline - (sprite.y as i16) & 0x07) as u16
                        } else {
                            // bottom half
                            ((sprite.tile_id as u16 & 0x01) << 12)
                                | (((sprite.tile_id & 0xFE) + 1) << 4) as u16
                                | (self.scanline - (sprite.y as i16) & 0x07) as u16
                        }
                    } else {
                        // sprite flipped vertically
                        if self.scanline - (sprite.y as i16) < 8 {
                            // top half
                            ((sprite.tile_id as u16 & 0x01) << 12)
                                | (((sprite.tile_id & 0xFE) + 1) << 4) as u16
                                | (7 - (self.scanline - (sprite.y as i16)) & 0x07) as u16
                        } else {
                            // bottom half
                            ((sprite.tile_id as u16 & 0x01) << 12)
                                | ((sprite.tile_id & 0xFE) << 4) as u16
                                | (7 - (self.scanline - (sprite.y as i16)) & 0x07) as u16
                        }
                    }
                } else {
                    // 8x8 mode
                    if sprite.attr & 0x80 == 0 {
                        // sprite normal
                        ((self.control.pattern_sprite as u16) << 12)
                            | ((sprite.tile_id as u16) << 4)
                            | (self.scanline as u16 - sprite.y as u16)
                    } else {
                        // sprite flipped vertically
                        ((self.control.pattern_sprite as u16) << 12)
                            | ((sprite.tile_id as u16) << 4)
                            | (7 - (self.scanline as u16 - sprite.y as u16))
                    }
                };

                let sprite_pattern_addr_hi = sprite_pattern_addr_lo + 8;

                let mut sprite_pattern_bits_lo = self.bus.read(sprite_pattern_addr_lo);
                let mut sprite_pattern_bits_hi = self.bus.read(sprite_pattern_addr_hi);

                if sprite.attr & 0x40 != 0 {
                    sprite_pattern_bits_lo = sprite_pattern_bits_lo.reverse_bits();
                    sprite_pattern_bits_hi = sprite_pattern_bits_hi.reverse_bits();
                }

                self.sprite_shifter_pattern_lo[i] = sprite_pattern_bits_lo;
                self.sprite_shifter_pattern_hi[i] = sprite_pattern_bits_hi;
            }
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
        self.status = Status::from(0x00);
        self.mask = Mask::from(0x00);
        self.control = Control::from(0x00);
        self.vram_address = VRAMAddress::from(0x0000);
        self.tram_address = VRAMAddress::from(0x0000);
    }
}

impl PPU {
    fn inc_x(&mut self) {
        if self.mask.render_background || self.mask.render_sprites {
            self.vram_address.coarse_x += 1;
            if self.vram_address.coarse_x == 32 {
                self.vram_address.coarse_x = 0;
                self.vram_address.nametable_x ^= 1;
            }
        }
    }

    fn inc_y(&mut self) {
        if self.mask.render_background || self.mask.render_sprites {
            if self.vram_address.fine_y < 7 {
                self.vram_address.fine_y += 1;
            } else {
                self.vram_address.fine_y = 0;
                if self.vram_address.coarse_y == 29 {
                    self.vram_address.coarse_y = 0;
                    self.vram_address.nametable_y ^= 1;
                } else if self.vram_address.coarse_y == 31 {
                    self.vram_address.coarse_y = 0;
                } else {
                    self.vram_address.coarse_y += 1;
                }
            }
        }
    }

    fn reset_x(&mut self) {
        if self.mask.render_background || self.mask.render_sprites {
            self.vram_address.nametable_x = self.tram_address.nametable_x;
            self.vram_address.coarse_x = self.tram_address.coarse_x;
        }
    }

    fn reset_y(&mut self) {
        if self.mask.render_background || self.mask.render_sprites {
            self.vram_address.nametable_y = self.tram_address.nametable_y;
            self.vram_address.coarse_y = self.tram_address.coarse_y;
            self.vram_address.fine_y = self.tram_address.fine_y;
        }
    }

    fn shift_shifters(&mut self) {
        if self.mask.render_background {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }

        if self.mask.render_sprites && self.cycle >= 1 && self.cycle < 258 {
            for (i, sprite) in self.scanline_sprites.iter_mut().enumerate() {
                if sprite.x == 0 {
                    self.sprite_shifter_pattern_lo[i] <<= 1;
                    self.sprite_shifter_pattern_hi[i] <<= 1;
                } else {
                    sprite.x -= 1;
                }
            }
        }
    }

    fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo =
            (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_shifter_pattern_hi =
            (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;

        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00)
            | if self.bg_next_tile_attrib & 0b01 == 0b01 {
                0xFF
            } else {
                0x00
            };
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00)
            | if self.bg_next_tile_attrib & 0b10 == 0b10 {
                0xFF
            } else {
                0x00
            };
    }
}

impl Device for PPU {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000 => 0x00,
            0x0001 => 0x00,
            0x0002 => {
                let data = u8::from(self.status) & 0xE0 | (self.buffer & 0x1F);
                self.status.vertical_blank = false;
                self.write_flip_flop = true;
                data
            }
            0x0003 => 0x00,
            0x0004 => self.oam.mem[self.oam.addr as usize],
            0x0005 => 0x00,
            0x0006 => 0x00,
            0x0007 => {
                let mut data = self.buffer;
                self.buffer = self.bus.read(u16::from(self.vram_address));

                if u16::from(self.vram_address) >= 0x3F00 {
                    data = self.buffer;
                    self.buffer = self.bus.read(u16::from(self.vram_address) - 0x1000);
                };

                let increment = if self.control.increment_mode { 32 } else { 1 };
                self.vram_address = (u16::from(self.vram_address) + increment).into();
                data
            }
            _ => 0x00,
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        match address {
            0x0000 => {
                self.control = Control::from(data);
                self.tram_address.nametable_x = self.control.nametable_x as u8;
                self.tram_address.nametable_y = self.control.nametable_y as u8;
            }
            0x0001 => {
                self.mask = Mask::from(data);
            }
            0x0002 => (),
            0x0003 => self.oam.addr = data,
            0x0004 => {
                self.oam.mem[self.oam.addr as usize] = data;
                self.oam.addr = self.oam.addr.wrapping_add(1);
            }
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
                    self.write_flip_flop = false;
                } else {
                    let addr: u16 = (u16::from(self.tram_address) & 0xFF00) | u16::from(data);
                    self.tram_address = addr.into();
                    self.vram_address = self.tram_address;
                    self.write_flip_flop = true;
                }
            }
            0x0007 => {
                let vram_address = u16::from(self.vram_address);

                self.bus.write(vram_address, data);

                // nametable index (0-3)
                let nametable_i = ((vram_address - 0x2000) / 0x400) % 4;
                match self.cartridge_mirror_mode {
                    //nametables: [A, A, B, B]
                    MirrorMode::Horizontal => {
                        if nametable_i == 0 || nametable_i == 2 {
                            self.bus.write(vram_address + 0x400, data);
                        } else if nametable_i == 1 || nametable_i == 3 {
                            self.bus.write(vram_address - 0x400, data);
                        };
                    }
                    //nametables: [A, B, A, B]
                    MirrorMode::Vertical => {
                        if nametable_i == 0 || nametable_i == 1 {
                            self.bus.write(vram_address + 0x800, data);
                        } else if nametable_i == 2 || nametable_i == 3 {
                            self.bus.write(vram_address - 0x800, data);
                        };
                    }
                }

                let increment = if self.control.increment_mode { 32 } else { 1 } as u16;
                self.vram_address = (u16::from(self.vram_address) + increment).into();
            }
            _ => (),
        }
    }
}
