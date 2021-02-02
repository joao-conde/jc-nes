use jc_nes::bus::Bus;
use jc_nes::cpu::CPU;
use jc_nes::ram::RAM;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use core::panic;
use std::{cell::RefCell, ops::Deref};
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
        .take(32 * 1024)
        .flatten()
        .collect::<Vec<u8>>(); // 16kB per bank
    let char_mem = bytes.by_ref().take(8 * 1024).flatten().collect::<Vec<u8>>(); // 8kB per bank

    let mut vram_mem = vec![0u8; 64 * 1024]; // 64kB VRAM
    &vram_mem[0x0000..0x2000].clone_from_slice(&char_mem);
    
    let width: usize = 128;
    let height: usize = 256;

    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem
        .window("Pattern Table", 3 *width as u32, 3 * height as u32)
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.clear();
    
    const TILE_PIXEL_WIDTH: usize = 8;
    const TILE_PIXEL_HEIGHT: usize = TILE_PIXEL_WIDTH;
    const TILE_BYTE_WIDTH: usize = 2 * TILE_PIXEL_WIDTH;

    let mut count_x = 1;
    let mut count_y = 1;
    for y in 0..height {
        for x in 0..width {
            let tile_x = x / TILE_PIXEL_WIDTH;
            let tile_y = y / TILE_PIXEL_HEIGHT;
            let addr = tile_y * height + tile_x * TILE_BYTE_WIDTH + (y % 8);

            let mut lsb: u8 = vram_mem[addr];
            let mut msb: u8 = vram_mem[addr + 8];

            let mut pixel_help: u16 = 0x0000;

            for i in 0..8 {
                let bit0: u8 = lsb & 0x01;
                let bit1: u8 = msb & 0x01;

                pixel_help |= (bit0 as u16) << (i * 2);
                pixel_help |= (bit1 as u16) << (i * 2 + 1);

                lsb >>= 1;
                msb >>= 1;
            }
            
            let pos = 7 - (x % 8);
            let opt = pos * 2;
            let pixel = (pixel_help & (0x3 << opt)) >> opt;
            // let pix_str = &format!("{}", pixel);
            // let pixel_ascii = if pixel == 0 { " " } else { pix_str };
            // print!("{} ", pixel_ascii);

            match pixel {
                0 => canvas.set_draw_color(Color::RGB(0, 0, 0)),
                1 => canvas.set_draw_color(Color::RGB(0, 102, 255)),
                2 => canvas.set_draw_color(Color::RGB(0, 51, 128)),
                3 => canvas.set_draw_color(Color::RGB(0, 10, 26)),
                _ => panic!("unexpected pixel value"),
            }

            for i in 0..3 {
                for j in 0..3 {
                    canvas.draw_point(
                        Point::new(i + 3 * x as i32, j + 3 * y as i32)
                    ).unwrap();
                }
            }
            
            if count_x % 8 == 0 {
                print!(" ");
            }
            count_x += 1;
        }
        if count_y % 8 == 0 {
            print!("\n");
        }
        count_y += 1;
        print!("\n");
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
