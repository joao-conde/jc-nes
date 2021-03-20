#[derive(Clone, Copy)]
pub struct Control {
    pub(in crate::ppu) nametable_x: bool,
    pub(in crate::ppu) nametable_y: bool,
    pub(in crate::ppu) increment_mode: bool,
    pub(in crate::ppu) pattern_sprite: bool,
    pub(in crate::ppu) pattern_background: bool,
    pub(in crate::ppu) sprite_size: bool,
    pub(in crate::ppu) slave_mode: bool,
    pub(in crate::ppu) enable_nmi: bool,
}

impl From<u8> for Control {
    fn from(byte: u8) -> Control {
        Control {
            nametable_x: (byte & 0x01) != 0,
            nametable_y: (byte & 0x02) != 0,
            increment_mode: (byte & 0x04) != 0,
            pattern_sprite: (byte & 0x08) != 0,
            pattern_background: (byte & 0x10) != 0,
            sprite_size: (byte & 0x20) != 0,
            slave_mode: (byte & 0x40) != 0,
            enable_nmi: (byte & 0x80) != 0,
        }
    }
}

impl From<Control> for u8 {
    fn from(control: Control) -> u8 {
        control.nametable_x as u8
            | (control.nametable_y as u8) << 1
            | (control.increment_mode as u8) << 2
            | (control.pattern_sprite as u8) << 3
            | (control.pattern_background as u8) << 4
            | (control.sprite_size as u8) << 5
            | (control.slave_mode as u8) << 6
            | (control.enable_nmi as u8) << 7
    }
}
