# Nintendo Entertainment System (NES) emulator

Nintendo Entertainment System (NES) emulator written in Rust.

This is the emulator crate itself, called `jc-nes`. It emulates the NES and exposes a public API to interact with it to retrieve screen pixels or audio data, and to provide gamepad/controller input.

## Main Modules

- `cpu`: Central Processing Unit (opcodes, instructions, memory address modes)
- `ppu`: Picture Processing Unit responsible for updating the screen with pixel data
- `bus`: 
  - abstracts communications between devices
  - devices can be connected with given address ranges
  - exposes a `read/write` interface that directs to the approriate device
  - takes care of address mirroring
- `cartridge`: metadata about the ROM/game inserted
- `cartridge/mappers`:
  - intercepts cartridge reads and writes and maps the addresses
  - different games use different mappers
  - there are 700+ mappers so some games might not run because of the missing mapper

<p align="center">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1TIk4afXnPGvEJpSsquIfG0Y_VuTPDeMl">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1UpJ0S0gQ-Ybjt4UOFihcRkUtUimmE0J3">
</p>

<p align="center">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1Jbl0HxsG46ijFEdCGPEi5WeebvkpZ8PA">
  <img width="40%" height="40%" align="center" src="https://drive.google.com/uc?export=view&id=1q-iU_ODlkV9vbK6A7YPtifDpRVPCGISE">
</p>

## Repository Contents

`core` contains the Rust emulator crate.

`frontends` contains multiple usages of this crate like:
- `frontends/desktop`: powered by SDL
- `frontends/web`: powered by WASM

# Running
## Desktop Application

```
$ cd frontends/desktop/
$ cargo run --release <ROM PATH>
```
