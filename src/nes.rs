use std::cell::RefCell;
use std::rc::Rc;
use crate::{bus::Bus, cartridge::Cartridge};
use crate::cpu::CPU;
use crate::ppu::PPU;
use crate::ram::RAM;

struct Nes<'a> {
    cpu: CPU<'a>,
    ppu: Rc<RefCell<PPU>>,
}

impl<'a> Nes<'a> {
    pub fn new() -> Nes<'a> {
        
        let ram = Rc::new(RefCell::new(RAM::new(vec![0u8; 2 * 1024])));
        
        let mut cpu_bus = Bus::default();
        let _ppu_bus = Bus::default();
        cpu_bus.connect(0x0000..=0x1FFF, &ram);
            
        let ppu = Rc::new(RefCell::new(PPU::new()));
        cpu_bus.connect_w(0x2000..=0x3FFF, &ppu);
        cpu_bus.add_mirror(0x0000..=0x1FFF, 0x07FF);
        cpu_bus.add_mirror(0x2000..=0x3FFF, 0x2008);

        let cpu = CPU::new(cpu_bus);

        Nes { cpu, ppu }
    }

    pub fn load_rom(&self, rom_path: &str) {

        let cartridge = Rc::new(RefCell::new(Cartridge::load_rom(rom_path)));

    }
}
