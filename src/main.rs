use jc_nes::cpu::CPU;
use jc_nes::ram::RAM;
use jc_nes::{bus::Bus, nes::Nes};
use sdl2::{
    keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect, render::Texture, surface::Surface,
};
use std::env;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::{cell::RefCell, time::Instant};

fn main() {
    let mode = env::args().nth(1).expect("No run mode specified");
    if mode == "nestest" {
        nestest()
    } else if mode == "play" {
        play(&env::args().nth(2).expect("No rom to play"))
    } else {
        panic!("Invalid mode");
    }
}

fn play(rom_path: &str) {
    let mut nes = Nes::new();
    nes.load_rom(rom_path);
    nes.reset();

    // SDL graphics
    const MAIN_WIDTH: u32 = 256;
    const MAIN_HEIGHT: u32 = 240;
    const MAIN_SCALING: u32 = 4;

    let sdl = sdl2::init().expect("failed to init SDL");
    let video_subsystem = sdl.video().expect("failed to get video context");

    let main_window = video_subsystem
        .window(rom_path, MAIN_WIDTH, MAIN_HEIGHT)
        .resizable()
        .build()
        .expect("failed to build window");
    let mut main_canvas = main_window
        .into_canvas()
        .build()
        .expect("failed to build window's canvas");
    // main_canvas
    //     .set_scale(MAIN_SCALING as f32, MAIN_SCALING as f32)
    //     .expect("failed setting window scale");
    main_canvas.clear();
    main_canvas.present();

    let texture_creator = main_canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    // emulate clock ticks
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        nes.clock();

        if nes.ppu.borrow().render {
            main_canvas.clear();
            // let now = Instant::now();
            // println!("RENDER: {}ns", now.elapsed().as_nanos());

            texture
                .update(None, &nes.ppu.borrow().screen, 256 * 3)
                .unwrap();
            main_canvas.copy(&texture, None, None).unwrap();

            main_canvas.present();
            nes.ppu.borrow_mut().render = false;
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
    }
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
    for _ in 0..26553 {
        cpu.clock();
    }
}
