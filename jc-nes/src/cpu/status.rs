#[derive(Clone, Copy, Default)]
pub struct Status {
    pub(in crate::cpu) carry: bool,
    pub(in crate::cpu) zero: bool,
    pub(in crate::cpu) interrupt: bool,
    pub(in crate::cpu) decimal: bool,
    pub(in crate::cpu) b1: bool,
    pub(in crate::cpu) b2: bool,
    pub(in crate::cpu) overflow: bool,
    pub(in crate::cpu) negative: bool,
}

impl From<u8> for Status {
    fn from(byte: u8) -> Status {
        Status {
            carry: (byte & 0x01) != 0,
            zero: (byte & 0x02) != 0,
            interrupt: (byte & 0x04) != 0,
            decimal: (byte & 0x08) != 0,
            b1: (byte & 0x10) != 0,
            b2: (byte & 0x20) != 0,
            overflow: (byte & 0x40) != 0,
            negative: (byte & 0x80) != 0,
        }
    }
}

impl From<Status> for u8 {
    fn from(status: Status) -> u8 {
        status.carry as u8
            | (status.zero as u8) << 1
            | (status.interrupt as u8) << 2
            | (status.decimal as u8) << 3
            | (status.b1 as u8) << 4
            | (status.b2 as u8) << 5
            | (status.overflow as u8) << 6
            | (status.negative as u8) << 7
    }
}
