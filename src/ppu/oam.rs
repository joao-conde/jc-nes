use crate::bus::Device;

pub struct OAM {
    oam_addr: u8,
    oam: [u8; 256],
    dma: bool,
}

impl OAM {
    pub fn read(&mut self) -> u8 {
        self.oam[self.oam_addr as usize]
    }

    pub fn write(&mut self, data: u8) {
        self.oam[self.oam_addr as usize] = data;
    }

    pub fn set_addr(&mut self, address: u8) {
        self.oam_addr = address;
    }
}

// This interface is exposed for DMA (0x4014 address)
impl Device for OAM {
    fn read(&mut self, address: u16) -> u8 {
        panic!("read OAMDMA");
        0x00
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
            oam_addr: 0x00,
            oam: [0u8; 256],
            dma: false,
        }
    }
}
