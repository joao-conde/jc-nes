use jc_nes::gamepad::Button;
use jc_nes::nes::Nes;
use jc_nes::ppu::{HEIGHT, WIDTH};
use sdl2::{keyboard::Keycode, pixels::PixelFormatEnum};
use std::{fs::File, io::Read};

const SCALE: f32 = 3.75;

fn main() {
    let sdl = sdl2::init().expect("failed to init SDL");
    let video_subsystem = sdl.video().expect("failed to get video context");

    let window = video_subsystem
        .window(
            "Drag and drop a ROM to play",
            SCALE as u32 * WIDTH as u32,
            SCALE as u32 * WIDTH as u32,
        )
        .resizable()
        .build()
        .expect("failed to build window");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("failed to build window's canvas");
    canvas
        .set_scale(SCALE, SCALE)
        .expect("failed setting window scale");
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    let mut nes = Nes::new();
    let mut game_loaded = false;

    // emulate clock ticks
    let mut timer_subsystem = sdl.timer().expect("failed to get timer system");
    let tick_interval = 1000 / 240; // frequency in Hz to period in ms
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

                sdl2::event::Event::DropFile { filename, .. } => {
                    game_loaded = true;
                    let rom = read_file(&filename);
                    nes = Nes::new();
                    nes.load_rom(&rom);
                    nes.reset();
                    None
                }

                sdl2::event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } if game_loaded => key_to_btn(keycode).map(|btn| nes.btn_down(1, btn)),

                sdl2::event::Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } if game_loaded => key_to_btn(keycode).map(|btn| nes.btn_up(1, btn)),

                _ => None,
            };
        }

        let current_time = timer_subsystem.ticks();
        let delta_t = current_time - last_update_time;
        if game_loaded && tick_interval > delta_t {
            // 1.79MHz / 60Hz
            (0..30).for_each(|_| nes.clock());
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

fn read_file(path: &str) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut rom = Vec::new();
    file.read_to_end(&mut rom).unwrap();
    rom
}
