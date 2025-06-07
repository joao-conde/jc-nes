use crate::bus::{Bus, Device, SharedMut, UnsafeDerefMut};
use crate::cartridge::mappers;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::gamepad::{Button, Gamepad};
use crate::ppu::dma::OamDma;
use crate::ppu::palette::Palette;
use crate::ppu::Ppu;
use crate::ram::Ram;
use std::cell::UnsafeCell;
use std::rc::Rc;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "web", wasm_bindgen)]
pub struct Nes {
    cpu: Cpu,
    ppu: SharedMut<Ppu>,
    dma_controller: SharedMut<OamDma>,
    gamepad1: SharedMut<Gamepad>,
    gamepad2: SharedMut<Gamepad>,
    cycles: usize,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
impl Nes {
    #[cfg_attr(feature = "web", wasm_bindgen(constructor))]
    pub fn new() -> Nes {
        // build PPU bus
        let mut ppu_bus = Bus::default();
        let nametbl1 = Ram::new(vec![0u8; 1024]);
        let nametbl2 = Ram::new(vec![0u8; 1024]);
        let nametbl3 = Ram::new(vec![0u8; 1024]);
        let nametbl4 = Ram::new(vec![0u8; 1024]);
        let palette = Palette::new();

        // connect (and mirror) devices to PPU bus
        ppu_bus.connect(0x2000..=0x23FF, nametbl1);
        ppu_bus.connect(0x2400..=0x27FF, nametbl2);
        ppu_bus.connect(0x2800..=0x2BFF, nametbl3);
        ppu_bus.connect(0x2C00..=0x2FFF, nametbl4);
        ppu_bus.connect(0x3F00..=0x3FFF, palette);
        ppu_bus.add_mirror(0x3000..=0x3EFF, 0x2EFF);
        ppu_bus.add_mirror(0x3F20..=0x3FFF, 0x3F1F);
        ppu_bus.add_mirror(0x4000..=0xFFFF, 0x3FFF);

        let ppu = Rc::new(UnsafeCell::new(Ppu::new(ppu_bus)));

        // build CPU bus
        let mut cpu_bus = Bus::default();
        let ram = Ram::new(vec![0u8; 2 * 1024]);
        let gamepad1 = Rc::new(UnsafeCell::new(Gamepad::default()));
        let gamepad2 = Rc::new(UnsafeCell::new(Gamepad::default()));
        let dma_controller = Rc::new(UnsafeCell::new(OamDma::default()));

        // connect (and mirror) devices to CPU bus
        cpu_bus.connect(0x0000..=0x1FFF, ram);
        cpu_bus.connect(0x2000..=0x3FFF, ppu.clone());
        cpu_bus.connect(0x4014..=0x4014, dma_controller.clone());
        cpu_bus.connect(0x4016..=0x4016, gamepad1.clone());
        cpu_bus.connect(0x4017..=0x4017, gamepad2.clone());

        // (APU address space and others)
        cpu_bus.connect(0x4000..=0x4013, Ram::new(vec![0u8; 32]));
        cpu_bus.connect(0x4015..=0x4015, Ram::new(vec![0u8; 32]));
        cpu_bus.connect(0x4018..=0x401F, Ram::new(vec![0u8; 32]));
        cpu_bus.connect(0x4020..=0x7FFF, Ram::new(vec![0u8; 15 * 1024]));

        // add mirrors
        cpu_bus.add_mirror(0x0000..=0x1FFF, 0x07FF);
        cpu_bus.add_mirror(0x2000..=0x3FFF, 0x2007);

        let cpu = Cpu::new(cpu_bus);

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
        self.ppu.inner().mirror_mode = cartridge.mirror;
        match cartridge.mapper_id {
            0 => self.connect_mapper(mappers::mapper000::new_mapper(cartridge)),
            3 => self.connect_mapper(mappers::mapper003::new_mapper(cartridge)),
            id => panic!("Unimplemented mapper {}", id),
        };
    }

    pub fn clock(&mut self) {
        self.ppu.inner().clock();

        if self.cycles % 3 == 0 {
            if self.dma_controller.inner().dma_in_progress {
                self.dma_controller
                    .inner()
                    .transfer(self.cycles, &mut self.cpu.bus);
            } else {
                self.cpu.clock();
            }
        }

        if self.ppu.inner().raise_nmi {
            self.ppu.inner().raise_nmi = false;
            self.cpu.nmi();
        }

        self.cycles += 1;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.inner().reset();
    }

    #[cfg(not(feature = "web"))]
    pub fn get_frame(
        &mut self,
    ) -> Option<[u8; crate::ppu::WIDTH as usize * crate::ppu::HEIGHT as usize * 3]> {
        if self.ppu.inner().frame_complete {
            self.ppu.inner().frame_complete = false;
            Some(self.ppu.inner().screen)
        } else {
            None
        }
    }

    #[cfg(feature = "web")]
    pub fn get_frame(&mut self) -> Option<Vec<u8>> {
        if self.ppu.inner().frame_complete {
            self.ppu.inner().frame_complete = false;
            Some(self.ppu.inner().screen.to_vec())
        } else {
            None
        }
    }

    pub fn btn_down(&mut self, controller: u8, btn: Button) {
        match controller {
            1 => self.gamepad1.inner().btn_down(btn),
            2 => self.gamepad2.inner().btn_down(btn),
            _ => eprintln!("expected either controller 1 or 2"),
        }
    }

    pub fn btn_up(&mut self, controller: u8, btn: Button) {
        match controller {
            1 => self.gamepad1.inner().btn_up(btn),
            2 => self.gamepad2.inner().btn_up(btn),
            _ => panic!("expected either controller '1' or '2'"),
        }
    }

    fn connect_mapper(
        &mut self,
        (prg_mapper, chr_mapper): (impl Device + 'static, impl Device + 'static),
    ) {
        self.cpu.bus.connect(0x8000..=0xFFFF, prg_mapper);
        self.ppu.inner().bus.connect(0x0000..=0x1FFF, chr_mapper);
    }
}

impl Default for Nes {
    fn default() -> Nes {
        Nes::new()
    }
}
