use crate::device::Device;
use crate::ram::Ram;
use crate::ppu::palette::Palette;
use crate::ppu::dma::OamDma;

pub struct PpuBus {
    pub(crate) dma_controller: OamDma,
    nametbl1: Ram,
    nametbl2: Ram,
    nametbl3: Ram,
    nametbl4: Ram,
    palette: Palette,
}

impl PpuBus {
    pub fn new() -> PpuBus {
        PpuBus {
            dma_controller: OamDma::default(),
            nametbl1: Ram::new(vec![0u8; 1024]),
            nametbl2: Ram::new(vec![0u8; 1024]),
            nametbl3: Ram::new(vec![0u8; 1024]),
            nametbl4: Ram::new(vec![0u8; 1024]),
            palette: Palette::new()
        }
    }

    fn get_device(&mut self, address: u16) -> &mut dyn Device {
        match address {
            // ppu
            0x2000..=0x23FF => &mut self.nametbl1,
            0x2400..=0x27FF => &mut self.nametbl2,
            0x2800..=0x2BFF => &mut self.nametbl3,
            0x2C00..=0x2FFF => &mut self.nametbl4,
            0x3F00..=0x3FFF => &mut self.palette,
            
            // mirrors 
            0x3000..=0x3EFF => self.get_device(address & 0x2EFF),
            0x3F20..=0x3FFF => self.get_device(address & 0x3F1F),
            0x4000..=0xFFFF => self.get_device(address & 0x3FFF),

            0x4014..=0x4014 => &mut self.dma_controller,
        
            x => unreachable!("read address: {:#04X}", x)
        }
    }
}

impl Device for PpuBus {
    fn read(&mut self, address: u16) -> u8 {
        let device = self.get_device(address);
        device.read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        let mut device = self.get_device(address);
        device.write(address, data)
    }
}
