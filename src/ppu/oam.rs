use crate::bus::Device;

pub struct OAM {
    pub(in crate::ppu) addr: usize,
    pub(in crate::ppu) mem: [u8; 256],
    dma: bool,
}

// This interface is exposed for DMA (address $4014)
impl Device for OAM {
    fn read(&mut self, address: u16) -> u8 {
        unimplemented!("can not read from OAMDMA ($4014)");
    }

    fn write(&mut self, address: u16, data: u8) {
        println!("write OAMDMA");
        self.dma = true;
        // todo!();
    }
}

impl Default for OAM {
    fn default() -> OAM {
        OAM {
            addr: 0x00,
            mem: [0u8; 256],
            dma: false,
        }
    }
}
