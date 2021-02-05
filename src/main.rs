use core::panic;
use jc_nes::cartridge::mappers::{mapper000::Mapper000, MapperMemoryPin};
use jc_nes::cartridge::Cartridge;
use jc_nes::cpu::CPU;
use jc_nes::ppu::PPU;
use jc_nes::ram::RAM;
use jc_nes::{bus::Bus, nes::Nes};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::time::Duration;

fn main() {
    //nestest();
    //emulate();
    play();
}

fn nestest() {
    // read test rom
    let mut rom = File::open("roms/nestest.nes").unwrap();
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).expect("buffer overflow");

    // make test rom address start at 0xC000
    // and discard 16-bit header
    let mut mem = Vec::new();
    (0..0xC000).for_each(|_| mem.push(0));
    buffer[16..0x4F00].iter().for_each(|byte| mem.push(*byte));

    // connect ram to the bus
    // give bus to CPU to read/write
    let ram = Rc::new(RefCell::new(RAM::new(mem)));
    let mut bus = Bus::default();
    bus.connect(0x0000..=0xFFFF, &ram);

    let mut cpu = CPU::new(bus);

    // emulate clock cycle
    for _ in 0..26548 {
        cpu.clock();
    }
}

fn play() {
    let rom_path = "roms/secret/donkey-kong.nes";

    let mut nes = Nes::new();
    nes.load_rom(rom_path);

    // SDL graphics
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
    const SCALING: u32 = 3;

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window(rom_path, SCALING * WIDTH, SCALING * HEIGHT)
        .resizable()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_scale(SCALING as f32, SCALING as f32).unwrap();
    canvas.clear();
    canvas.present();

    // emulate clock ticks, CPU 3x slower than PPU
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        nes.clock();
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
        }
    }
}

fn emulate() {
    let rom_path = "roms/secret/donkey-kong.nes";
    let cartridge = Cartridge::load_rom(rom_path);
    let cartridge = Rc::new(RefCell::new(cartridge));

    let mapper_cpu = Mapper000::new(MapperMemoryPin::PrgROM, &cartridge, 2);
    let mapper_cpu = Rc::new(RefCell::new(mapper_cpu));

    let mapper_ppu = Mapper000::new(MapperMemoryPin::ChrROM, &cartridge, 2);
    let mapper_ppu = Rc::new(RefCell::new(mapper_ppu));

    let mut ppu_bus = Bus::default();
    ppu_bus.connect(0x0000..=0x1FFF, &mapper_ppu);

    let ppu = Rc::new(RefCell::new(PPU::new(ppu_bus)));

    let ram = Rc::new(RefCell::new(RAM::new(vec![0u8; 2 * 1024])));
    let mut cpu_bus = Bus::default();
    cpu_bus.connect_writable(0x2000..=0x3FFF, &ppu);
    cpu_bus.connect(0x4020..=0xFFFF, &mapper_cpu);
    cpu_bus.connect(0x0000..=0x1FFF, &ram);

    cpu_bus.add_mirror(0x0000..=0x1FFF, 0x07FF);
    cpu_bus.add_mirror(0x2000..=0x3FFF, 0x2008);

    let mut cpu = CPU::new(cpu_bus);

    // SDL graphics
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
    const SCALING: u32 = 3;

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window(rom_path, SCALING * WIDTH, SCALING * HEIGHT)
        .resizable()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_scale(SCALING as f32, SCALING as f32).unwrap();
    canvas.clear();
    canvas.present();

    // emulate clock ticks, CPU 3x slower than PPU
    let mut i: usize = 0;
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        ppu.borrow_mut().clock();
        if i % 3 == 0 {
            cpu.clock();
        }

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
        }

        i += 1;
    }
}

fn display_pattern_table(ppu_bus: &Bus) {
    const WIDTH: u32 = 128;
    const HEIGHT: u32 = 256;
    const SCALING: u32 = 3;

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem
        .window("Pattern Table", SCALING * WIDTH, SCALING * HEIGHT)
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_scale(SCALING as f32, SCALING as f32).unwrap();
    canvas.clear();

    const TILE_PIXEL_WIDTH: u32 = 8;
    const TILE_PIXEL_HEIGHT: u32 = TILE_PIXEL_WIDTH;
    const TILE_BYTE_WIDTH: u32 = 2 * TILE_PIXEL_WIDTH;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // get base address of pixel
            let tile_x = x / TILE_PIXEL_WIDTH;
            let tile_y = y / TILE_PIXEL_HEIGHT;
            let pixel_y = y % 8;
            let addr = tile_y * HEIGHT + tile_x * TILE_BYTE_WIDTH + pixel_y;

            // get data from both bit planes
            let mut lsb: u8 = ppu_bus.read(addr as u16);
            let mut msb: u8 = ppu_bus.read(addr as u16 + 8);

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

    let mut render = true;
    let mut event_pump = sdl.event_pump().unwrap();
    while render {
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => render = false,
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => render = false,
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
