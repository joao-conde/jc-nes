use crate::device::Device;
use crate::gamepad::Gamepad;
use crate::ram::Ram;

pub struct CpuBus {
    ram: Ram,
    pub(crate) prg_mapper: Option<Box<dyn Device>>,
    pub(crate) chr_mapper: Option<Box<dyn Device>>,
    pub(crate) gamepad1: Gamepad,
    pub(crate) gamepad2: Gamepad
}

impl CpuBus {
    pub fn new() -> CpuBus {
        CpuBus {
            ram: Ram::new(vec![0u8; 2 * 1024]),
            gamepad1: Gamepad::default(),
            gamepad2: Gamepad::default(),
            prg_mapper: None,
            chr_mapper: None
        }
    }

    fn get_device(&mut self, address: u16) -> &mut dyn Device {
        match address {
            // cpu
            0x0000..=0x1FFF => &mut self.ram,

            // mirrors 
            0x3000..=0x3EFF => self.get_device(address & 0x2EFF),
            0x3F20..=0x3FFF => self.get_device(address & 0x3F1F),
            0x4000..=0xFFFF => self.get_device(address & 0x3FFF),
            
            // dma
            
            0x4016..=0x4016 => &mut self.gamepad1,
            0x4017..=0x4017 => &mut self.gamepad2,

            // mappers
            0x8000..=0xFFFF => {
                let prg_mapper = self.prg_mapper.as_mut().unwrap().as_mut();    
                prg_mapper
            },
            0x0000..=0x1FFF => {
                let chr_mapper: &mut dyn Device = self.chr_mapper.as_mut().unwrap().as_mut();
                chr_mapper
            },

            x => unreachable!("read address: {:#04X}", x)
        }
    }
}

impl Device for CpuBus {
    fn read(&mut self, address: u16) -> u8 {
        let device = self.get_device(address);
        device.read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        let mut device = self.get_device(address);
        device.write(address, data)
    }
}
