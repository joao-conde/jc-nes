#[derive(Clone, Copy)]
pub struct Mask {
    pub(in crate::ppu) gray_scale: bool,
    pub(in crate::ppu) render_background_left: bool,
    pub(in crate::ppu) render_sprites_left: bool,
    pub(in crate::ppu) render_background: bool,
    pub(in crate::ppu) render_sprites: bool,
    pub(in crate::ppu) enhance_red: bool,
    pub(in crate::ppu) enhance_green: bool,
    pub(in crate::ppu) enhance_blue: bool,
}

impl From<u8> for Mask {
    fn from(byte: u8) -> Mask {
        Mask {
            gray_scale: (byte & 0x01) != 0,
            render_background_left: (byte & 0x02) != 0,
            render_sprites_left: (byte & 0x04) != 0,
            render_background: (byte & 0x08) != 0,
            render_sprites: (byte & 0x10) != 0,
            enhance_red: (byte & 0x20) != 0,
            enhance_green: (byte & 0x40) != 0,
            enhance_blue: (byte & 0x80) != 0,
        }
    }
}

impl From<Mask> for u8 {
    fn from(mask: Mask) -> u8 {
        mask.gray_scale as u8
            | (mask.render_background_left as u8) << 1
            | (mask.render_sprites_left as u8) << 2
            | (mask.render_background as u8) << 3
            | (mask.render_sprites as u8) << 4
            | (mask.enhance_red as u8) << 5
            | (mask.enhance_green as u8) << 6
            | (mask.enhance_blue as u8) << 7
    }
}
