use crate::bus::Device;
use crate::cartridge::mappers;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gamepad::{Button, Gamepad};
use crate::ppu::Ppu;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "web", wasm_bindgen)]
pub struct Nes {
    cpu: Cpu,
    ppu: Rc<RefCell<Ppu>>,
    gamepad1: Gamepad,
    gamepad2: Gamepad,
    cycles: usize,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl Nes {
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    pub fn new() -> Nes {
        let ppu = Rc::new(RefCell::new(Ppu::new()));
        let cpu = Cpu::new(ppu.clone());
        let gamepad1 = Gamepad::default();
        let gamepad2 = Gamepad::default();
        Nes {
            cpu,
            ppu,
            gamepad1,
            gamepad2,
            cycles: 0,
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let cartridge = Cartridge::new(rom);
        self.ppu.borrow_mut().mirror_mode = cartridge.mirror;
        match cartridge.mapper_id {
            0 => self.connect_mapper(mappers::mapper000::new_mapper(cartridge)),
            3 => self.connect_mapper(mappers::mapper003::new_mapper(cartridge)),
            id => panic!("Unimplemented mapper {}", id),
        };
    }

    pub fn clock(&mut self) {
        self.ppu.borrow_mut().clock();

        if self.cycles % 3 == 0 {
            if self.dma_controller.dma_in_progress {
                self.dma_controller.transfer(self.cycles, &mut self.cpu.bus);
            } else {
                self.cpu.clock();
            }
        }

        let mut ppu = self.ppu.borrow_mut();
        if ppu.raise_nmi {
            ppu.raise_nmi = false;
            self.cpu.nmi();
        }

        self.cycles += 1;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.borrow_mut().reset();
    }

    #[cfg(not(feature = "web"))]
    pub fn get_frame(
        &mut self,
    ) -> Option<[u8; crate::ppu::WIDTH as usize * crate::ppu::HEIGHT as usize * 3]> {
        let mut ppu = self.ppu.borrow_mut();

        if ppu.frame_complete {
            ppu.frame_complete = false;
            Some(ppu.screen)
        } else {
            None
        }
    }

    #[cfg(feature = "web")]
    pub fn get_frame(&mut self) -> Option<Vec<u8>> {
        if self.ppu.borrow().frame_complete {
            self.ppu.borrow_mut().frame_complete = false;
            Some(self.ppu.borrow().screen.to_vec())
        } else {
            None
        }
    }

    pub fn btn_down(&mut self, controller: u8, btn: Button) {
        match controller {
            1 => self.gamepad1.btn_down(btn),
            2 => self.gamepad2.btn_down(btn),
            _ => eprintln!("expected either controller 1 or 2"),
        }
    }

    pub fn btn_up(&mut self, controller: u8, btn: Button) {
        match controller {
            1 => self.gamepad1.btn_up(btn),
            2 => self.gamepad2.btn_up(btn),
            _ => panic!("expected either controller '1' or '2'"),
        }
    }

    fn connect_mapper(
        &mut self,
        (prg_mapper, chr_mapper): (impl Device + Clone + 'static, impl Device + Clone + 'static),
    ) {
        // self.bus.connect(0x8000..=0xFFFF, prg_mapper);
        // self.bus.connect(0x0000..=0x1FFF, chr_mapper);

        let prg_mapper = Box::new(prg_mapper);
        let chr_mapper = Box::new(chr_mapper);

        self.cpu.bus.prg_mapper = Some(prg_mapper.clone());
        self.cpu.bus.chr_mapper = Some(chr_mapper.clone());

        let mut ppu = self.ppu.borrow_mut();
        ppu.bus.prg_mapper = Some(prg_mapper);
        ppu.bus.chr_mapper = Some(chr_mapper);
    }
}

impl Default for Nes {
    fn default() -> Nes {
        Nes::new()
    }
}
