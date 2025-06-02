use crate::bus::Device;
use crate::ram::Ram;

pub struct Bus {
    ram: Ram,
    pub prg_mapper: Option<Box<dyn Device>>,
    pub ppu_state: [u8; 8],
    pub ppu_diff: Option<PpuDiff>,
}

pub enum PpuDiff {
    Read { address: u16 },
    Write { address: u16, data: u8 },
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: Ram::new(vec![0u8; 2 * 1024]),
            prg_mapper: None,
            ppu_state: [0u8; 8],
            ppu_diff: None,
        }
    }

    pub fn connect_prg_mapper(&mut self, mapper: impl Device + 'static) {
        self.prg_mapper = Some(Box::new(mapper));
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            // CPU RAM and mirror
            0x0000..0x0800 => self.ram.read(address),
            0x0800..0x2000 => self.read(address % 0x0800),

            // PPU and mirror
            0x2000..0x2008 => {
                self.ppu_diff = Some(PpuDiff::Read { address });
                self.ppu_state[address as usize - 0x2000]
            }
            0x2008..0x4000 => self.read(address % 0x8),

            0x8000..=0xFFFF => self
                .prg_mapper
                .as_mut()
                .map(|mapper| mapper.read(address - 0x8000))
                .unwrap(),

            _ => panic!("out of bounds 0x{:08x}", address),
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            // CPU RAM and mirror
            0x0000..0x0800 => self.ram.write(address, data),
            0x0800..0x2000 => self.write(address % 0x0800, data),

            // PPU and mirror
            0x2000..0x2008 => {
                self.ppu_diff = Some(PpuDiff::Write { address, data });
                self.ppu_state[address as usize - 0x2000] = data
            }

            0x2008..0x4000 => self.write(address % 0x8, data),

            0x8000..=0xFFFF => todo!("CPU WRITE TO PROGRAM MAPPER"),

            _ => panic!("out of bounds 0x{:08x}", address),
        };
    }
}
