use crate::bus::{Bus, Device};

pub struct OAMDMA {
    pub(in crate) dma_in_progress: bool,
    synched: bool,
    buffer: u8,
    page: u8,
    transfered: u8,
}

impl Default for OAMDMA {
    fn default() -> OAMDMA {
        OAMDMA {
            dma_in_progress: false,
            synched: false,
            buffer: 0x00,
            page: 0x00,
            transfered: 0x00,
        }
    }
}

impl OAMDMA {
    pub fn transfer(&mut self, cur_cyc: usize, cpu_bus: &mut Bus) {
        if self.synched {
            if cur_cyc % 2 == 0 {
                // read byte from mem based on page
                let address = (self.page as u16) << 8 | self.transfered as u16;
                self.buffer = cpu_bus.read(address);
            } else {
                // write buffered byte to OAMDATA ($2004)
                cpu_bus.write(0x2004, self.buffer);

                // increment transfered count and stop if
                // 256 bytes were transfered (count overflow)
                self.transfered = self.transfered.wrapping_add(1);
                if self.transfered == 0x00 {
                    self.stop();
                }
            }
        } else {
            // next cycle is even and can start
            self.synched = cur_cyc % 2 == 1;
        }
    }

    fn start(&mut self, page: u8) {
        self.dma_in_progress = true;
        self.synched = false;
        self.buffer = 0x00;
        self.transfered = 0x00;
        self.page = page;
    }

    fn stop(&mut self) {
        self.dma_in_progress = false;
        self.synched = false;
        self.buffer = 0x00;
        self.transfered = 0x00;
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
