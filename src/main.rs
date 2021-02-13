use jc_nes::cpu::CPU;
use jc_nes::ram::RAM;
use jc_nes::{bus::Bus, nes::Nes};
use sdl2::keyboard::Keycode;
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

fn main() {
    if let Some(mode) = env::args().nth(1) {
        if mode == "nestest" {
            nestest()
        } else if mode == "play" {
            play()
        }
    } else {
        play()
    }
}

fn nestest() {
    // read test rom
    let mut rom = File::open("roms/nestest.nes").unwrap();
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).expect("buffer OVERFLOW");

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

fn play() {
    let rom_path = "roms/secret/donkey-kong.nes";
    // let rom_path = "roms/full_palette.nes";
    // let rom_path = "roms/full_palette_alt.nes";
    // let rom_path = "roms/nestest.nes";

    let mut nes = Nes::new();
    nes.load_rom(rom_path);
    nes.reset();

    // SDL graphics
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
    const SCALING: u32 = 4;

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
        nes.clock(&mut canvas);
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,

                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => nes.draw_name_table(0, &mut canvas, 32, 30),

                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => nes.draw_name_table(1, &mut canvas, 32, 30),

                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => nes.draw_name_table(2, &mut canvas, 32, 30),

                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => nes.draw_name_table(3, &mut canvas, 32, 30),

                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => nes.draw_pattern_table(&mut canvas, WIDTH, HEIGHT),

                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => nes.draw_screen(&mut canvas, WIDTH as usize, HEIGHT as usize),

                _ => {}
            }
        }
    }
}
