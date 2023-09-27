use crate::device::Device;
use crate::cartridge::mappers;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gamepad::Button;
use crate::ppu::{Ppu, HEIGHT, WIDTH};

pub struct Nes {
    cpu: Cpu,
    cycles: usize,
}

impl Nes {
    pub fn new() -> Nes {
        let ppu = Ppu::new();
        let cpu = Cpu::new(ppu);
        Nes {
            cpu,
            cycles: 0,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let cartridge = Cartridge::new(rom);
        self.cpu.bus.ppu.mirror_mode = cartridge.mirror;
        match cartridge.mapper_id {
            0 => self.connect_mapper(mappers::mapper000::new_mapper(cartridge)),
            3 => self.connect_mapper(mappers::mapper003::new_mapper(cartridge)),
            id => panic!("Unimplemented mapper {}", id),
        };
    }

    pub fn clock(&mut self) {
        self.cpu.bus.ppu.clock();

        let ppu = &mut self.cpu.bus.ppu;

        if self.cycles % 3 == 0 {
            if ppu.bus.dma_controller.dma_in_progress {
                ppu.bus.dma_controller.transfer(self.cycles, &mut self.cpu.bus);
            } else {
                self.cpu.clock();
            }
        }

        if self.cpu.bus.ppu.raise_nmi {
            self.cpu.bus.ppu.raise_nmi = false;
            self.cpu.nmi();
        }

        self.cycles += 1;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cpu.bus.ppu.reset();
    }

    pub fn get_frame(&mut self) -> Option<[u8; WIDTH as usize * HEIGHT as usize * 3]> {
        if self.cpu.bus.ppu.frame_complete {
            self.cpu.bus.ppu.frame_complete = false;
            Some(self.cpu.bus.ppu.screen)
        } else {
            None
        }
    }

    pub fn btn_down(&mut self, controller: u8, btn: Button) {
        match controller {
            1 => self.cpu.bus.gamepad1.btn_down(btn),
            2 => self.cpu.bus.gamepad2.btn_down(btn),
            _ => eprintln!("expected either controller '1' or '2'"),
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
        self.cpu.bus.ppu.bus.chr_mapper = Some(Box::new(chr_mapper));
    }
}

impl Default for Nes {
    fn default() -> Nes {
        Nes::new()
    }
}
