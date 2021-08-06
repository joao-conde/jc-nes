# Nintendo Entertainment System (NES) emulator

Nintendo Entertainment System (NES) emulator written in Rust.

<p align="center">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1TIk4afXnPGvEJpSsquIfG0Y_VuTPDeMl">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1UpJ0S0gQ-Ybjt4UOFihcRkUtUimmE0J3">
</p>

<p align="center">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1Jbl0HxsG46ijFEdCGPEi5WeebvkpZ8PA">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1q-iU_ODlkV9vbK6A7YPtifDpRVPCGISE">
</p>

# Running

See the [`examples`](./examples) folder.

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

Typical usage:

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

# Contributing

Each game uses a specific mapper, and there are a ton of them. I will not implement them all. Incoming PRs with new mapper implementations are welcome. Check the [`mappers` source code module](./src/cartridge/mappers) to see the current implementation and the [NESDev Wiki](https://wiki.nesdev.com/w/index.php/Mapper) for more information.
