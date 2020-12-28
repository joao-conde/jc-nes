# Documentation

This folder contains summary information about the NES console compiled from the [NES dev wiki](http://wiki.nesdev.com/), youtube tutorials and by analyzing open-source emulators.

# Main devices

## [CPU](http://wiki.nesdev.com/w/index.php/CPU)

- Central Processing Unit
- based on the MOS6502 processor 
- runs at approximately 1.79 MHz 
- made by Ricoh and lacks the MOS6502's decimal mode
- 64kB addressable range (0x0000 - 0xFFFF)
- [registers](http://wiki.nesdev.com/w/index.php/CPU_registers):
    - 1 byte each
    - A: accumulator 
    - X: x
    - Y: y
    - PC: program counter
    - SP: stack pointer
    - SR: status register

## [PPU](http://wiki.nesdev.com/w/index.php/PPU)

- Picture Processing Unit
- 0x2000 - 0x2007
- 16kB addressable range (0x00 - 0xFF)
- outputs pixel at each clock
- 3x the clock speed of CPU
- bus of its own
 
# Connected to CPU bus

## [APU](http://wiki.nesdev.com/w/index.php/APU)

- Audio Processing Unit
- 0x4000 - 0x4017
- generates sound
- part of the CPU in reality

## RAM

- Random Access Memory
- 0x0000 - 0x07FF
- 2kB

# Connected to PPU bus

## Graphics (game tiles, part of the Cartridge)

- 8kB (0x0000 - 0x1FFF)

## VRAM

- 2kB (0x2000 - 0x27FF)

## Palettes

- 0x3F00 - 0x3FFF
- 2 bits per pixel index into palette

## OAM

- location of sprites
- transfer of data from CPU to OAM with DMA (direct memory access)

## Cartridge

- ROM with program
- 0x4020 - 0xFFFF
- contains a Mapper

## Mappers

- responsible for bank switching (cpu configures mapper to switch banks/data ranges on the cartridge)
- cpu/ppu still address the same range on cartridge but mapper points to different data/banks
