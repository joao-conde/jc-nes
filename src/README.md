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