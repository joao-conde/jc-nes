use crate::cpu::CPU;
use crate::ppu::PPU;
use crate::ram::RAM;
use crate::{
    bus::Bus,
    cartridge::{
        mappers::{mapper000::Mapper000, MapperMemoryPin},
        Cartridge,
    },
};
use std::cell::RefCell;
use std::rc::Rc;

pub type SharedMut<T> = Rc<RefCell<T>>;

pub struct Nes<'a> {
    cpu: CPU<'a>,
    ppu: SharedMut<PPU<'a>>,
    ticks: usize,
}

impl<'a> Nes<'a> {
    pub fn new() -> Nes<'a> {
        // PPU bus devices
        // let nametbl = Rc::new(RefCell::new(NameTable::new(vec![0u8; 2 * 1024])));
        // let palette = Rc::new(RefCell::new(Palette::new(vec![0u8; 2 * 1024])));

        // Connect devices to PPU bus
        let mut ppu_bus = Bus::default();
        // ppu_bus.connect(0x2000..=0x2FFF, &nametbl);
        // ppu_bus.connect(0x3F00..=0x3FFF, &palette);
        let ppu = Rc::new(RefCell::new(PPU::new(ppu_bus)));

        // CPU bus devices
        let ram = Rc::new(RefCell::new(RAM::new(vec![0u8; 2 * 1024])));

        // Connect devices to CPU bus
        let mut cpu_bus = Bus::default();
        cpu_bus.connect(0x0000..=0x1FFF, &ram);
        cpu_bus.connect(0x2000..=0x3FFF, &ppu);
        cpu_bus.add_mirror(0x0000..=0x1FFF, 0x07FF);
        cpu_bus.add_mirror(0x2000..=0x3FFF, 0x2007);
        let cpu = CPU::new(cpu_bus);

        Nes { cpu, ppu, ticks: 0 }
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        let cartridge = Cartridge::load_rom(rom_path);
        let cartridge = Rc::new(RefCell::new(cartridge));

        let header = cartridge.borrow().header;

        match header.mapper_id {
            0 => {
                let mapper_cpu = Mapper000::new(MapperMemoryPin::PrgROM, &cartridge, 2);
                let mapper_cpu = Rc::new(RefCell::new(mapper_cpu));
                self.cpu.bus.connect(0x4020..=0xFFFF, &mapper_cpu);

                let mapper_ppu = Mapper000::new(MapperMemoryPin::ChrROM, &cartridge, 2);
                let mapper_ppu = Rc::new(RefCell::new(mapper_ppu));
                self.ppu
                    .borrow_mut()
                    .bus
                    .connect(0x0000..=0x1FFF, &mapper_ppu);
            }
            _ => panic!("unknown mapper!"),
        }
    }

    pub fn clock(&mut self) {
        self.ppu.borrow_mut().clock();
        if self.ticks % 3 == 0 {
            self.cpu.clock();
        }
        self.ticks += 1;
    }
}

impl<'a> Default for Nes<'a> {
    fn default() -> Self {
        Self::new()
    }
}
