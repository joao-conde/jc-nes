use crate::device::Device;
use crate::gamepad::Gamepad;
use crate::ppu::Ppu;
use crate::ram::Ram;

pub struct CpuBus {
    ram: Ram,
    pub(crate) prg_mapper: Option<Box<dyn Device>>,
    pub(crate) gamepad1: Gamepad,
    pub(crate) gamepad2: Gamepad,
    others: Ram,
    pub(crate) ppu: Ppu,
}

impl CpuBus {
    pub fn new(ppu: Ppu) -> CpuBus {
        CpuBus {
            ram: Ram::new(vec![0u8; 2 * 1024]),
            gamepad1: Gamepad::default(),
            gamepad2: Gamepad::default(),
            prg_mapper: None,
            ppu: ppu,
            others: Ram::new(vec![0u8; 32 + 32 + 32 + (15 * 1024)])
        }
    }

    fn mirror(&self, address: u16) -> u16 {
        match address {
            0x0000..=0x1FFF => address & 0x07FF,
            0x2000..=0x3FFF => address & 0x2007,
            address => address
        }
    }

    fn get_device(&mut self, address: u16) -> &mut dyn Device {
        let address = self.mirror(address);
        let prg_mapper = self.prg_mapper.as_mut().unwrap();

        match address {
            0x8000..=0xFFFF => prg_mapper,

            0x0000..=0x07FF => &mut self.ram,
            // 0x2000..=0x3FFF => ppu.clone()),
            // 0x4014..=0x4014 => &mut self.dma_controller,
            0x4016..=0x4016 => &mut self.gamepad1,
            0x4017..=0x4017 => &mut self.gamepad2,

            // (APU address space and others)
            0x4000..=0x4013 => &mut self.others,
            0x4015..=0x4015 => &mut self.others,
            0x4018..=0x401F => &mut self.others,
            0x4020..=0x7FFF => &mut self.others,

            x => unreachable!("read address: {:#04X}", x)
        }
    }
}

impl Device for CpuBus {
    fn read(&mut self, address: u16) -> u8 {
        println!("read from {:#04X}", address);
        let device = self.get_device(address);
        device.read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        let device = self.get_device(address);
        device.write(address, data)
    }
}
