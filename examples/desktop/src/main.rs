use jc_nes::{Button, Nes, HEIGHT, WIDTH};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};
use std::{fs::File, io::Read};

const SYSTEM_HZ: u32 = 240;
const SCREEN_SCALE: f32 = 3.75;
const TITLE: &str = "Drag and drop the ROM file to play";

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let mut timer_subsystem = sdl.timer().unwrap();

    let window = video_subsystem
        .window(
            TITLE,
            SCREEN_SCALE as u32 * WIDTH as u32,
            SCREEN_SCALE as u32 * HEIGHT as u32,
        )
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_scale(SCREEN_SCALE, SCREEN_SCALE).unwrap();
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    let mut nes = Nes::new();
    let mut game_loaded = false;

    let tick_interval = 1000 / SYSTEM_HZ;
    let mut last_update_time = 0;
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,

                Event::DropFile { filename, .. } => {
                    let rom = read_file(&filename);
                    nes = Nes::new();
                    nes.load_rom(&rom);
                    nes.reset();
                    game_loaded = true;
                    canvas
                        .window_mut()
                        .set_title(&format!("{} [Currently playing: {}]", TITLE, filename))
                        .unwrap();
                    None
                }

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } if game_loaded => key_to_btn(keycode).map(|btn| nes.btn_down(1, btn)),

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } if game_loaded => key_to_btn(keycode).map(|btn| nes.btn_up(1, btn)),

                _ => None,
            };
        }

        let current_time = timer_subsystem.ticks();
        let delta_t = current_time - last_update_time;
        if game_loaded && tick_interval > delta_t {
            // 1.79MHz / 240Hz
            (0..8).for_each(|_| nes.clock());
            if let Some(screen) = nes.get_frame() {
                timer_subsystem.delay(tick_interval - delta_t);
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
