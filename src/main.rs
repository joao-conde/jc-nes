use core::panic;
use jc_nes::bus::Bus;
use jc_nes::cpu::CPU;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::fs::File;
use std::io::Read;
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

    let _prg_rom = bytes
        .by_ref()
        .take(32 * 1024)
        .flatten()
        .collect::<Vec<u8>>(); // 16kB per bank

    let char_rom = bytes.by_ref().take(8 * 1024).flatten().collect::<Vec<u8>>(); // 8kB per bank

    let mut ppu_mem = vec![0u8; 64 * 1024]; // 64kB PPU RAM
    ppu_mem[0x0000..0x2000].clone_from_slice(&char_rom);

    let ppu_bus = Bus::new(ppu_mem);
    display_pattern_table(&ppu_bus);
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

    let mut bus = Bus::new(mem);
    let mut cpu = CPU::new(&mut bus);

    // emulate clock cycle
    for _ in 0..26548 {
        cpu.clock();
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
