use crate::bus::{Bus, SharedMut};
use crate::cartridge::mappers::mapper000::{CHRMapper000, PRGMapper000};
use crate::cartridge::mappers::mapper002::{CHRMapper002, PRGMapper002};
use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::gamepad::{Button, Gamepad};
use crate::ppu::dma::OAMDMA;
use crate::ppu::palette::Palette;
use crate::ppu::{HEIGHT, PPU, WIDTH};
use crate::ram::RAM;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Nes {
    cpu: CPU,
    ppu: SharedMut<PPU>,
    dma_controller: SharedMut<OAMDMA>,
    gamepad1: SharedMut<Gamepad>,
    gamepad2: SharedMut<Gamepad>,
    cycles: usize,
}

impl Nes {
    pub fn new() -> Nes {
        // build PPU bus
        let mut ppu_bus = Bus::default();
        let nametbl1 = RAM::new(vec![0u8; 1024]);
        let nametbl2 = RAM::new(vec![0u8; 1024]);
        let nametbl3 = RAM::new(vec![0u8; 1024]);
        let nametbl4 = RAM::new(vec![0u8; 1024]);
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

        let ppu = Rc::new(RefCell::new(PPU::new(ppu_bus)));

        // build CPU bus
        let mut cpu_bus = Bus::default();
        let ram = RAM::new(vec![0u8; 2 * 1024]);
        let controller1 = Rc::new(RefCell::new(Gamepad::default()));
        let controller2 = Rc::new(RefCell::new(Gamepad::default()));
        let dma_controller = Rc::new(RefCell::new(OAMDMA::default()));

        // connect (and mirror) devices to CPU bus
        cpu_bus.connect(0x0000..=0x1FFF, ram);
        cpu_bus.connect(0x2000..=0x3FFF, ppu.clone());
        cpu_bus.connect(0x4014..=0x4014, dma_controller.clone());
        cpu_bus.connect(0x4016..=0x4016, controller1.clone());
        cpu_bus.connect(0x4017..=0x4017, controller2.clone());
        // TODO remove temporary memory fillers (APU or IO related)
        cpu_bus.connect(0x4000..=0x4013, RAM::new(vec![0u8; 32]));
        cpu_bus.connect(0x4015..=0x4015, RAM::new(vec![0u8; 32]));
        cpu_bus.connect(0x4018..=0x401F, RAM::new(vec![0u8; 32]));
        //
        cpu_bus.add_mirror(0x0000..=0x1FFF, 0x07FF);
        cpu_bus.add_mirror(0x2000..=0x3FFF, 0x2007);

        let cpu = CPU::new(cpu_bus);

        Nes {
            cpu,
            ppu,
            dma_controller,
            gamepad1: controller1,
            gamepad2: controller2,
            cycles: 0,
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        let cartridge = Cartridge::new(rom_path);
        self.ppu.borrow_mut().cartridge_mirror_mode = cartridge.mirror;
        match cartridge.mapper_id {
            0 => {
                let prg_mapper = PRGMapper000::new(cartridge.prg_rom, cartridge.prg_banks);
                self.cpu.bus.connect(0x8000..=0xFFFF, prg_mapper);

                let chr_mapper = CHRMapper000::new(cartridge.chr_rom, cartridge.chr_banks);
                self.ppu
                    .borrow_mut()
                    .bus
                    .connect(0x0000..=0x1FFF, chr_mapper);
            },
            2 => {
                let prg_mapper = PRGMapper002::new(cartridge.prg_rom, cartridge.prg_banks);
                self.cpu.bus.connect(0x8000..=0xFFFF, prg_mapper);

                let chr_mapper = CHRMapper002::new(cartridge.chr_rom, cartridge.chr_banks);
                self.ppu
                    .borrow_mut()
                    .bus
                    .connect(0x0000..=0x1FFF, chr_mapper);
            }
            id => panic!("unknown mapper {}", id),
        }
    }

    pub fn clock(&mut self) {
        self.ppu.borrow_mut().clock();

        if self.cycles % 3 == 0 {
            if self.dma_controller.borrow().dma_in_progress {
                self.dma_controller
                    .borrow_mut()
                    .transfer(self.cycles, &mut self.cpu.bus);
            } else {
                self.cpu.clock();
            }
        }

        if self.ppu.borrow().raise_nmi {
            self.ppu.borrow_mut().raise_nmi = false;
            self.cpu.nmi();
        }

        self.cycles += 1;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.borrow_mut().reset();
    }

    pub fn get_frame(&mut self) -> Option<[u8; WIDTH as usize * HEIGHT as usize * 3]> {
        if self.ppu.borrow().frame_complete {
            self.ppu.borrow_mut().frame_complete = false;
            Some(self.ppu.borrow().screen)
        } else {
            None
        }
    }

    pub fn btn_down(&mut self, controller: u8, btn: Button) {
        match controller {
            1 => self.gamepad1.borrow_mut().btn_down(btn),
            2 => self.gamepad2.borrow_mut().btn_down(btn),
            _ => eprintln!("expected either controller 1 or 2"),
        }
    }

    pub fn btn_up(&mut self, controller: u8, btn: Button) {
        match controller {
            1 => self.gamepad1.borrow_mut().btn_up(btn),
            2 => self.gamepad2.borrow_mut().btn_up(btn),
            _ => panic!("expected either controller '1' or '2'"),
        }
    }
}
