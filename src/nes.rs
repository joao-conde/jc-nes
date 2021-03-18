use crate::bus::Bus;
use crate::cartridge::{
    mappers::{mapper000::Mapper000, MapperMemoryPin},
    Cartridge,
};
use crate::cpu::CPU;
use crate::ppu::PPU;
use crate::ram::RAM;
use std::cell::RefCell;
use std::rc::Rc;

pub type SharedMut<T> = Rc<RefCell<T>>;

pub struct Nes<'a> {
    cpu: CPU<'a>,
    pub ppu: SharedMut<PPU<'a>>,
    ticks: usize,
}

impl<'a> Nes<'a> {
    pub fn new() -> Nes<'a> {
        // PPU bus devices
        let nametbl1 = Rc::new(RefCell::new(RAM::new(vec![0u8; 1024])));
        let nametbl2 = Rc::new(RefCell::new(RAM::new(vec![0u8; 1024])));
        let nametbl3 = Rc::new(RefCell::new(RAM::new(vec![0u8; 1024])));
        let nametbl4 = Rc::new(RefCell::new(RAM::new(vec![0u8; 1024])));
        let palette = Rc::new(RefCell::new(RAM::new(vec![0u8; 256])));

        let mut ppu_bus = Bus::default();
        ppu_bus.connect(0x2000..=0x23FF, &nametbl1);
        ppu_bus.connect(0x2400..=0x27FF, &nametbl2);
        ppu_bus.connect(0x2800..=0x2BFF, &nametbl3);
        ppu_bus.connect(0x2C00..=0x2FFF, &nametbl4);
        ppu_bus.connect(0x3F00..=0x3FFF, &palette);
        ppu_bus.add_mirror(0x3000..=0x3EFF, 0x2EFF);
        ppu_bus.add_mirror(0x4000..=0xFFFF, 0x3FFF);

        let ppu = Rc::new(RefCell::new(PPU::new(ppu_bus)));

        // CPU bus devices
        let ram = Rc::new(RefCell::new(RAM::new(vec![0u8; 2 * 1024])));
        let tmp = Rc::new(RefCell::new(RAM::new(vec![0u8; 32]))); // TODO: remove tmp hack (IO + APU)

        let mut cpu_bus = Bus::default();
        cpu_bus.connect(0x0000..=0x1FFF, &ram);
        cpu_bus.connect(0x2000..=0x3FFF, &ppu);
        cpu_bus.connect(0x4000..=0x401F, &tmp);
        cpu_bus.add_mirror(0x0000..=0x1FFF, 0x07FF);
        cpu_bus.add_mirror(0x2000..=0x3FFF, 0x2007);

        let mut cpu = CPU::new(cpu_bus);
        cpu.debug = false;

        Nes { cpu, ppu, ticks: 0 }
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        let cartridge = Cartridge::new(rom_path);
        let cartridge = Rc::new(RefCell::new(cartridge));

        let meta = cartridge.borrow().meta.clone();
        match meta.mapper_id {
            0 => {
                let mapper_cpu = Mapper000::new(MapperMemoryPin::PrgROM, &cartridge);
                let mapper_cpu = Rc::new(RefCell::new(mapper_cpu));
                self.cpu.bus.connect(0x8000..=0xFFFF, &mapper_cpu);

                let mapper_ppu = Mapper000::new(MapperMemoryPin::ChrROM, &cartridge);
                let mapper_ppu = Rc::new(RefCell::new(mapper_ppu));
                self.ppu
                    .borrow_mut()
                    .bus
                    .connect(0x0000..=0x1FFF, &mapper_ppu);

                self.ppu.borrow_mut().set_mirror(meta.mirror);
            }
            _ => panic!("unknown mapper!"),
        }
    }

    pub fn clock(&mut self) {
        self.ppu.borrow_mut().clock();
        if self.ticks % 3 == 0 {
            self.cpu.clock();
        }

        if self.ppu.borrow().raise_nmi {
            self.ppu.borrow_mut().raise_nmi = false;
            self.cpu.nmi();
        }

        self.ticks += 1;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.borrow_mut().reset();
    }
}
