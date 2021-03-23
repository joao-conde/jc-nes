pub struct OAM {
    pub(in crate::ppu) addr: usize,
    pub(in crate::ppu) mem: [u8; 256],
}

impl Default for OAM {
    fn default() -> OAM {
        OAM {
            addr: 0x00,
            mem: [0u8; 256],
        }
    }
}

pub struct Sprite {
    pub(in crate::ppu) y: u8,
    pub(in crate::ppu) x: u8,
    pub(in crate::ppu) tile_id: u8,
    pub(in crate::ppu) attr: u8,
}

impl From<&[u8]> for Sprite {
    fn from(bytes: &[u8]) -> Sprite {
        Sprite {
            y: bytes[0],
            x: bytes[3],
            tile_id: bytes[1],
            attr: bytes[2],
        }
    }
}
