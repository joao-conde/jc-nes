use crate::device::Device;
use crate::ram::Ram;
use crate::ppu::palette::Palette;
use crate::ppu::dma::OamDma;

pub struct PpuBus {
    pub(crate) dma_controller: OamDma,
    pub(crate) chr_mapper: Option<Box<dyn Device>>,
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
            chr_mapper: None,
            nametbl1: Ram::new(vec![0u8; 1024]),
            nametbl2: Ram::new(vec![0u8; 1024]),
            nametbl3: Ram::new(vec![0u8; 1024]),
            nametbl4: Ram::new(vec![0u8; 1024]),
            palette: Palette::new()
        }
    }

    fn mirror(&self, address: u16) -> u16 {
        match address {
            0x3000..=0x3EFF => address & 0x2EFF,
            0x3F20..=0x3FFF => address & 0x3F1F,
            0x4000..=0xFFFF => address & 0x3FFF,
            address => address,
        }
    }

    fn get_device(&mut self, address: u16) -> (u16, &mut dyn Device) {
        let address = self.mirror(address);

        let chr_mapper = self.chr_mapper.as_mut().unwrap();

        match address {
            0x0000..=0x1FFF => (0x0000, chr_mapper),

            0x2000..=0x23FF => (0x2000, &mut self.nametbl1),
            0x2400..=0x27FF => (0x2400, &mut self.nametbl2),
            0x2800..=0x2BFF => (0x2800, &mut self.nametbl3),
            0x2C00..=0x2FFF => (0x2C00, &mut self.nametbl4),
            0x3F00..=0x3F1F => (0x3F00, &mut self.palette),

            0x4014..=0x4014 => (0x4014, &mut self.dma_controller),
            
            x => unreachable!("read address: {:#04X}", x)
        }
    }
}

impl Device for PpuBus {
    fn read(&mut self, address: u16) -> u8 {
        let (offset, device) = self.get_device(address);
        device.read(address - offset)
    }

    fn write(&mut self, address: u16, data: u8) {
        let (offset, device) = self.get_device(address);
        device.write(address - offset, data)
    }
}
