use crate::bus::Device;
use crate::cartridge::mappers;
use crate::cartridge::Cartridge;
use crate::cpu::bus::PpuDiff;
use crate::cpu::Cpu;
use crate::gamepad::{Button, Gamepad};
use crate::ppu::dma::OamDma;
use crate::ppu::Ppu;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "web", wasm_bindgen)]
pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    dma_controller: OamDma,
    gamepad1: Gamepad,
    gamepad2: Gamepad,
    cycles: usize,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl Nes {
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    pub fn new() -> Nes {
        let ppu = Ppu::new();
        let cpu = Cpu::new();
        let dma_controller = OamDma::default();
        let gamepad1 = Gamepad::default();
        let gamepad2 = Gamepad::default();
        Nes {
            cpu,
            ppu,
            dma_controller,
            gamepad1,
            gamepad2,
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
            if self.dma_controller.dma_in_progress {
                self.dma_controller.transfer(self.cycles, &mut self.cpu.bus);
            } else {
                self.cpu.clock();

                match self.cpu.bus.ppu_diff {
                    Some(PpuDiff::Read { address }) => {
                        self.ppu.read(address - 0x2000);
                    }
                    Some(PpuDiff::Write { address, data }) => {
                        self.ppu.write(address - 0x2000, data);
                    }
                    None => (),
                }
                self.cpu.bus.ppu_diff = None;
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

    #[cfg(not(feature = "web"))]
    pub fn get_frame(
        &mut self,
    ) -> Option<[u8; crate::ppu::WIDTH as usize * crate::ppu::HEIGHT as usize * 3]> {
        if self.ppu.frame_complete {
            self.ppu.frame_complete = false;
            Some(self.ppu.screen)
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
        self.cpu.bus.connect_prg_mapper(prg_mapper);
        self.ppu.bus.connect_chr_mapper(chr_mapper);
    }
}

impl Default for Nes {
    fn default() -> Nes {
        Nes::new()
    }
}
