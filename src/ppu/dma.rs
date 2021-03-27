use crate::bus::{Bus, Device};
use crate::ppu::oam::OAM;

pub struct OAMDMA {
    pub dma_in_progress: bool,
    synched: bool,
    buffer: u8,
    page: u8,
    transfered: usize,
}

impl Default for OAMDMA {
    fn default() -> OAMDMA {
        OAMDMA {
            dma_in_progress: false,
            synched: false,
            buffer: 0x00,
            page: 0x00,
            transfered: 0,
        }
    }
}

impl OAMDMA {
    pub fn transfer(&mut self, cur_cyc: usize, bus: &mut Bus, oam: &mut OAM) {
        if self.synched {
            if cur_cyc % 2 == 0 {
                let lo = self.transfered as u8 + oam.addr as u8;
                let from = (self.page as u16) << 8 | lo as u16;
                self.buffer = bus.read(from);
            } else {
                oam.mem[self.transfered] = self.buffer;
                self.transfered += 1;
                if self.transfered == 256 {
                    self.stop();
                }
            }
        } else {
            self.synched = cur_cyc % 2 == 1;
        }
    }

    fn start(&mut self, page: u8) {
        self.dma_in_progress = true;
        self.synched = false;
        self.buffer = 0x00;
        self.transfered = 0;
        self.page = page;
    }

    fn stop(&mut self) {
        self.dma_in_progress = false;
        self.synched = false;
        self.buffer = 0x00;
        self.transfered = 0;
        self.page = 0x00;
    }
}

// This interface is exposed for OAMDMA (address $4014 on CPU Bus)
impl Device for OAMDMA {
    fn read(&mut self, _address: u16) -> u8 {
        eprintln!("can not read from OAMDMA ($4014)");
        0x00
    }

    fn write(&mut self, _address: u16, data: u8) {
        self.start(data);
    }
}
