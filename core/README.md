# NES Emulator

This is the core crate called `jc-nes`. It takes care of all emulator related stuff and exposes a public API to interact with it to retrieve screen pixels or sound information and feed controller.

Multiple frontends can be used with this core like:
- browser APIS (if you compile it to WASM and use `canvas`, `Audio`, `onkeydown/up`, ...)
- desktop media libraries (like `SDL`, `SFML`)
- mobile apps (using for example the `Android NDK`)

At the moment there is only one frontend, built with SDL and targetting desktops.

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
  - there are 700+ mappers
  - by implementing mappers 0, 1, 2 and 3 most of the NES games are covered

## Missing Features

- mappers 1 & 2
- Audio Processing Unit (APU)
- name table mirroring for `ONESCREEN_LO` and `ONESCREEN_HI`
- bubble up errors to the library user rather than `panic!`

## Known Bugs

- `KungFu` does not start
- `Super Mario Brothers` visual corruption, random pixels
