#[derive(Clone, Copy)]
pub struct Status {
    pub(in crate::ppu) spr_overflow: bool,
    pub(in crate::ppu) spr_zero_hit: bool,
    pub(in crate::ppu) vertical_blank: bool,
}

impl From<u8> for Status {
    fn from(byte: u8) -> Status {
        Status {
            spr_overflow: (byte & 0x20) != 0,
            spr_zero_hit: (byte & 0x40) != 0,
            vertical_blank: (byte & 0x80) != 0,
        }
    }
}

impl From<Status> for u8 {
    fn from(status: Status) -> u8 {
        (status.spr_overflow as u8) << 5
            | (status.spr_zero_hit as u8) << 6
            | (status.vertical_blank as u8) << 7
    }
}
