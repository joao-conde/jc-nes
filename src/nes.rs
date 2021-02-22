use sdl2::{pixels::Color, rect::Point, render::Canvas, video::Window};

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
    ppu: SharedMut<PPU<'a>>,
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

        let cpu = CPU::new(cpu_bus);

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

    pub fn clock(&mut self, canvas: &mut Canvas<Window>) {
        self.ppu.borrow_mut().clock();
        if self.ticks % 3 == 0 {
            self.cpu.clock();
        }

        if self.ppu.borrow().raise_nmi {
            self.ppu.borrow_mut().raise_nmi = false;
            self.cpu.nmi();
        }

        if self.ppu.borrow().render {
            self.draw_screen(canvas, 256, 240);
            self.ppu.borrow_mut().render = false;
        }

        self.ticks += 1;
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.ppu.borrow_mut().reset();
    }

    pub fn draw_screen(&self, canvas: &mut Canvas<Window>, width: usize, height: usize) {
        canvas.clear();
        for y in 0..height {
            for x in 0..width {
                let (r, g, b) = self.ppu.borrow().screen[y][x];
                canvas.set_draw_color(Color::RGB(r, g, b));
                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
        }
        canvas.present();
    }

    pub fn draw_name_table(
        &self,
        table: usize,
        canvas: &mut Canvas<Window>,
        width: usize,
        height: usize,
    ) {
        canvas.clear();

        for y in 0..height {
            for x in 0..width {
                let address = (0x2000 + 0x400 * table) + x + y * width;
                let byte = self.ppu.borrow().bus.read(address as u16);
                print!("{:02X}", byte);
                // self.draw_tile(canvas, byte, x as i32 * 8, y as i32 * 8);
            }
            println!()
        }

        canvas.present();
    }

    pub fn draw_pattern_table(&self, canvas: &mut Canvas<Window>, width: u32, height: u32) {
        canvas.clear();

        const TILE_PIXEL_WIDTH: u32 = 8;
        const TILE_PIXEL_HEIGHT: u32 = TILE_PIXEL_WIDTH;
        const TILE_BYTE_WIDTH: u32 = 2 * TILE_PIXEL_WIDTH;

        for y in 0..height {
            for x in 0..width {
                // get base address of pixel
                let tile_x = x / TILE_PIXEL_WIDTH;
                let tile_y = y / TILE_PIXEL_HEIGHT;

                let pixel_y = y % 8;
                let addr = tile_y * height + tile_x * TILE_BYTE_WIDTH + pixel_y;

                // get data from both bit planes
                let mut lsb: u8 = self.ppu.borrow_mut().bus.read(addr as u16);
                let mut msb: u8 = self.ppu.borrow_mut().bus.read(addr as u16 + 8);

                // join bit plane data
                let mut pixel_help: u16 = 0x0000;
                for i in 0..8 {
                    let bit0: u8 = lsb & 0x01;
                    let bit1: u8 = msb & 0x01;

                    pixel_help |= (bit0 as u16) << (i * 2);
                    pixel_help |= (bit1 as u16) << (i * 2 + 1);

                    lsb >>= 1;
                    msb >>= 1;
                }

                // compute pixel number (from 0 to 3)
                let pos = 7 - (x % 8);
                let opt = pos * 2;
                let pixel = (pixel_help & (0x3 << opt)) >> opt;

                // draw
                match pixel {
                    0 => canvas.set_draw_color(Color::RGB(0, 0, 0)),
                    1 => canvas.set_draw_color(Color::RGB(0, 102, 255)),
                    2 => canvas.set_draw_color(Color::RGB(0, 51, 128)),
                    3 => canvas.set_draw_color(Color::RGB(0, 10, 26)),
                    _ => panic!("unexpected pixel value"),
                }
                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
        }
        canvas.present();
    }
}
