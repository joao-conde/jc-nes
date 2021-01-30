use jc_nes::bus::Bus;
use jc_nes::cpu::CPU;
use jc_nes::ram::RAM;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::time::Duration;

fn main() {
    dev()
    // nestest()
}

fn dev() {
    let rom_path = "roms/secret/donkey-kong.nes";
    let mut file = File::open(rom_path).unwrap();
    let mut rom = Vec::new();
    file.read_to_end(&mut rom).unwrap();
    // skip header (16 bytes)
    let mut bytes = rom.bytes().skip(16);
    // need to copy this to CPU RAM:
    let _prg_mem = bytes
        .by_ref()
        .take(16 * 1024)
        .flatten()
        .collect::<Vec<u8>>(); // 16kB per bank
    let char_mem = bytes.by_ref().take(8 * 1024).flatten().collect::<Vec<u8>>(); // 8kB per bank

    let mut vram = vec![0u8; 64 * 1024]; // 64kB VRAM
    vram[0x0000..0x2000].clone_from_slice(&char_mem);
    let vram = Rc::new(RefCell::new(RAM { mem: vram }));

    let mut bus = Bus::default();
    bus.connect(0x0000..=0xFFFF, &vram);

    let width = 128;
    let height = 256;

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem
        .window("Pattern Table", width, height)
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.clear();
    for r in 0..height {
        for c in 0..width {
            let addr = (r / 8 * 0x100) + (r % 8) + (c / 8) * 0x10;
            let v1 = bus.read(addr as u16).unwrap();
            let v2 = bus.read(addr as u16 + 8).unwrap();
            let pixel = ((v1 >> (7 - (c % 8))) & 0b1) + ((v2 >> (7 - (c % 8))) & 0b1) * 2;
            match pixel {
                0 => canvas.set_draw_color(Color::RGB(128, 179, 255)),
                1 => canvas.set_draw_color(Color::RGB(0, 102, 255)),
                2 => canvas.set_draw_color(Color::RGB(0, 51, 128)),
                3 => canvas.set_draw_color(Color::RGB(0, 10, 26)),
                _ => canvas.set_draw_color(Color::RGB(0, 0, 0)),
            }
            canvas.draw_point(Point::new(c as i32, r as i32)).unwrap();
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

fn nestest() {
    // read test rom
    let mut rom = File::open("roms/nestest.nes").unwrap();
    let mut buffer = [0u8; 64 * 1024];
    rom.read(&mut buffer).expect("buffer overflow");

    // make test rom address start at 0xC000
    // and discard 16-bit header
    let mut mem = Vec::new();
    (0..0xC000).for_each(|_| mem.push(0));
    buffer[16..0x4F00].iter().for_each(|byte| mem.push(*byte));

    // connect ram to the bus
    // give bus to CPU to read/write
    let ram = Rc::new(RefCell::new(RAM { mem }));
    let mut bus = Bus::default();
    bus.connect(0x0000..=0xFFFF, &ram);

    let mut cpu = CPU::new(&mut bus);

    // emulate clock cycle
    for _ in 0..26548 {
        cpu.clock();
    }
}
