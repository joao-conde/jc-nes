use crate::bus::{Bus, Device};
use crate::ppu::oam::OAM;

pub struct OAMDMA {
    pub dma_in_progress: bool,
    synched: bool,
    buffer: u8,
    page: u8,
    addr: u8,
}

impl Default for OAMDMA {
    fn default() -> OAMDMA {
        OAMDMA {
            dma_in_progress: false,
            synched: false,
            buffer: 0x00,
            page: 0x00,
            addr: 0x00,
        }
    }
}

impl OAMDMA {
    pub fn transfer(&mut self, cur_cyc: usize, bus: &Bus, oam: &mut OAM) {
        if self.synched {
            if cur_cyc % 2 == 0 {
                self.buffer = bus.read((self.page as u16) << 8 | self.addr as u16);
            } else {
                oam.mem[self.addr as usize] = self.buffer;

                self.addr = self.addr.wrapping_add(1);
                if self.addr == 0x00 {
                    self.dma_in_progress = false;
                    self.synched = false;
                }
            }
        } else {
            self.synched = cur_cyc % 2 == 1;
        }
    }
}

// This interface is exposed for OAMDMA (address $4014 on CPU Bus)
impl Device for OAMDMA {
    fn read(&mut self, _address: u16) -> u8 {
        panic!("can not read from OAMDMA ($4014)");
    }

    fn write(&mut self, _address: u16, _data: u8) {
        self.dma_in_progress = true;
    }
}
