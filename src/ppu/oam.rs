use crate::bus::{Bus, Device};
use crate::ppu::SharedMut;
use std::rc::Rc;

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
