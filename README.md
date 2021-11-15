# Nintendo Entertainment System (NES) emulator

Nintendo Entertainment System (NES) emulator written in Rust.

<p align="center">
  <img width="40%" height="40%" align="center" src="https://user-images.githubusercontent.com/16060539/135888265-476158f3-48a2-47b2-b115-1457616983f1.gif">
  <img width="40%" height="40%" align="center" src="https://user-images.githubusercontent.com/16060539/135888276-6051768f-b0e0-4f45-91ae-8f5fd954d607.gif">
</p>

<p align="center">
  <img width="40%" height="40%" align="center" src="https://user-images.githubusercontent.com/16060539/135888259-7c8f5907-d9e7-4e70-8379-995c164ea55e.gif">
  <img width="40%" height="40%" align="center" src="https://user-images.githubusercontent.com/16060539/135888282-949c9667-b140-4276-9aaa-88cd14615687.gif">
</p>

# Running

See the [`bin`](./bin) folder.

# API Reference

Use the `Nes` struct to create an emulator instance and interact with it using the following API:

```rust
impl Nes {
  pub fn new() -> Nes;
  pub fn load_rom(&mut self, rom: &[u8]);
  pub fn reset(&mut self);
  pub fn clock(&mut self);
  pub fn get_frame(&mut self) -> Option<[u8; SCREEN_WIDTH * SCREEN_HEIGHT * 3]>;
  pub fn btn_down(&mut self, controller: u8, btn: Button);
  pub fn btn_up(&mut self, controller: u8, btn: Button);
}
```

Basic usage:

```rust
use jc_nes::{Button, Nes, SCREEN_HEIGHT, SCREEN_WIDTH};

let mut nes = Nes::new();
nes.load_rom(&rom);
nes.reset();

loop {
  nes.clock();

  // Your draw code
  if let Some(screen) = nes.get_frame() {
    ...
  }

  // Your event processing
  match event {
    ... => nes.btn_down(1, Button::Up)
    ... => nes.btn_down(1, Button::B)
    ... => nes.btn_up(1, Button::A)
    ... => nes.btn_up(2, Button::Down)
    ...
  }
}
```
