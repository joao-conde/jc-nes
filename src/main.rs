use jc_nes::cpu::CPU;
use jc_nes::gamepad::Button;
use jc_nes::ppu::{HEIGHT, WIDTH};
use jc_nes::ram::RAM;
use jc_nes::{bus::Bus, nes::Nes};
use sdl2::{
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    render::{Canvas, Texture},
    video::Window,
    Sdl,
};
use std::fs::File;
use std::io::Read;

const SCALE: f32 = 4.0;

fn main() {
    // play("roms/nestest.nes");
    play("roms/ignored/pacman.nes");
}

fn play(rom_path: &str) {
    let mut nes = Nes::new();
    nes.load_rom(rom_path);
    nes.reset();

    let sdl = sdl2::init().expect("failed to init SDL");
    let video_subsystem = sdl.video().expect("failed to get video context");

    let main_window = video_subsystem
        .window(
            rom_path,
            SCALE as u32 * WIDTH as u32,
            SCALE as u32 * WIDTH as u32,
        )
        .resizable()
        .build()
        .expect("failed to build window");
    let mut main_canvas = main_window
        .into_canvas()
        .build()
        .expect("failed to build window's canvas");
    main_canvas
        .set_scale(SCALE, SCALE)
        .expect("failed setting window scale");
    main_canvas.clear();
    main_canvas.present();

    let texture_creator = main_canvas.texture_creator();

    let texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    play_60fps(nes, sdl, texture, main_canvas);
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
    let ram = RAM::new(mem);
    let mut bus = Bus::default();
    bus.connect(0x0000..=0xFFFF, ram);

    let mut cpu = CPU::new(bus);

    // emulate clock cycle
    for _ in 0..26553 {
        cpu.clock();
    }
}

fn play_60fps(mut nes: Nes, sdl: Sdl, mut texture: Texture, mut canvas: Canvas<Window>) {
    // emulate clock ticks
    let mut timer_subsystem = sdl.timer().expect("failed to get timer system");
    let tick_interval = 1000 / 120; // frequency in Hz to period in ms
    let mut last_update_time = 0;
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        while let Some(event) = event_pump.poll_event() {
            match event {
                // close window
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,

                // key down
                sdl2::event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => key_to_btn(keycode).map(|btn| nes.btn_down(1, btn)),

                // key up
                sdl2::event::Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => key_to_btn(keycode).map(|btn| nes.btn_up(1, btn)),

                _ => None,
            };
        }

        let current_time = timer_subsystem.ticks();
        let delta_t = current_time - last_update_time;

        if tick_interval > delta_t {
            // 1.79MHz / 60Hz
            for _ in 0..30 {
                nes.clock();
            }
            if let Some(screen) = nes.get_frame() {
                timer_subsystem.delay(tick_interval - delta_t); // energy saving
                texture.update(None, &screen, WIDTH as usize * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
            }
        }

        last_update_time = current_time;
    }
}

fn key_to_btn(keycode: Keycode) -> Option<Button> {
    match keycode {
        Keycode::Up => Some(Button::Up),
        Keycode::Down => Some(Button::Down),
        Keycode::Left => Some(Button::Left),
        Keycode::Right => Some(Button::Right),
        Keycode::A => Some(Button::A),
        Keycode::S => Some(Button::B),
        Keycode::Z => Some(Button::Start),
        Keycode::X => Some(Button::Select),
        _ => None,
    }
}
