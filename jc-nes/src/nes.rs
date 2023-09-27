use std::cell::RefCell;
use std::rc::Rc;

use crate::device::{Device, SharedMut};
use crate::cartridge::mappers;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gamepad::Button;
use crate::ppu::{Ppu, HEIGHT, WIDTH};

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    cycles: usize,
}

impl Nes {
    pub fn new() -> Nes {
        // build PPU bus

        let ppu = Ppu::new();

        // build CPU bus

        // (APU address space and others)
        // cpu_bus.connect(0x4000..=0x4013, Ram::new(vec![0u8; 32]));
        // cpu_bus.connect(0x4015..=0x4015, Ram::new(vec![0u8; 32]));
        // cpu_bus.connect(0x4018..=0x401F, Ram::new(vec![0u8; 32]));
        // cpu_bus.connect(0x4020..=0x7FFF, Ram::new(vec![0u8; 15 * 1024]));

        // add mirrors
        // cpu_bus.add_mirror(0x0000..=0x1FFF, 0x07FF);
        // cpu_bus.add_mirror(0x2000..=0x3FFF, 0x2007);

        let cpu = Cpu::new();

        Nes {
            cpu,
            ppu,
            cycles: 0,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let cartridge = Cartridge::new(rom);
        self.ppu.mirror_mode = cartridge.mirror;
        match cartridge.mapper_id {
            0 => self.connect_mapper(mappers::mapper000::new_mapper(cartridge)),
            3 => self.connect_mapper(mappers::mapper003::new_mapper(cartridge)),
            id => panic!("Unimplemented mapper {}", id),
        };
    }

    pub fn clock(&mut self) {
        self.ppu.clock();

        if self.cycles % 3 == 0 {
            if self.ppu.bus.dma_controller.dma_in_progress {
                self.ppu.bus.dma_controller.transfer(self.cycles, &mut self.cpu.bus);
            } else {
                self.cpu.clock();
            }
        }

        if self.ppu.raise_nmi {
            self.ppu.raise_nmi = false;
            self.cpu.nmi();
        }

        self.cycles += 1;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.reset();
    }

    pub fn get_frame(&mut self) -> Option<[u8; WIDTH as usize * HEIGHT as usize * 3]> {
        if self.ppu.frame_complete {
            self.ppu.frame_complete = false;
            Some(self.ppu.screen)
        } else {
            None
        }
    }

    pub fn btn_down(&mut self, controller: u8, btn: Button) {
        match controller {
            1 => self.cpu.bus.gamepad1.btn_down(btn),
            2 => self.cpu.bus.gamepad2.btn_down(btn),
            _ => eprintln!("expected either controller 1 or 2"),
        }
    }

    pub fn btn_up(&mut self, controller: u8, btn: Button) {
        match controller {
            1 => self.cpu.bus.gamepad1.btn_up(btn),
            2 => self.cpu.bus.gamepad2.btn_up(btn),
            _ => panic!("expected either controller '1' or '2'"),
        }
    }

    fn connect_mapper(
        &mut self,
        (prg_mapper, chr_mapper): (impl Device + 'static, impl Device + 'static),
    ) {
        self.cpu.bus.prg_mapper = Some(Box::new(prg_mapper));
        self.cpu.bus.chr_mapper = Some(Box::new(chr_mapper));

        // self.cpu.bus.connect(0x8000..=0xFFFF, prg_mapper);
        // self.ppu
            
        //     .bus
        //     .connect(0x0000..=0x1FFF, chr_mapper);
    }
}

impl Default for Nes {
    fn default() -> Nes {
        Nes::new()
    }
}
