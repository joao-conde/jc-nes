#[derive(Clone, Copy, Debug, Default)]
pub(in crate::ppu) struct Address {
    pub(in crate::ppu) coarse_x: u8,
    pub(in crate::ppu) coarse_y: u8,
    pub(in crate::ppu) nametable_x: u8,
    pub(in crate::ppu) nametable_y: u8,
    pub(in crate::ppu) fine_y: u8,
}

impl From<u16> for Address {
    fn from(word: u16) -> Address {
        Address {
            coarse_x: (word & 0x001F) as u8,
            coarse_y: (word & 0x03E0) as u8,
            nametable_x: (word & 0x0400) as u8,
            nametable_y: (word & 0x0800) as u8,
            fine_y: (word & 0x7000) as u8,
        }
    }
}

impl From<Address> for u16 {
    fn from(address: Address) -> u16 {
        (address.coarse_x as u16
            | (address.coarse_y as u16) << 5
            | (address.nametable_x as u16) << 10
            | (address.nametable_y as u16) << 11
            | (address.fine_y as u16) << 12)
    }
}
