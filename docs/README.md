# Documentation

This folder contains summary information about the NES console compiled from the [NES dev wiki](http://wiki.nesdev.com/), youtube tutorials and by analyzing open-source emulators.

# Main devices

- [CPU](cpu.md)
- [PPU](ppu.md)

# TODO REMOVE

# Connected to CPU bus

## PPU Registers

- 0x2000-0x2007

## [APU](http://wiki.nesdev.com/w/index.php/APU)

- Audio Processing Unit
- 0x4000 - 0x4017
- generates sound
- part of the CPU in reality

## RAM

- Random Access Memory
- 0x0000 - 0x1FFF
- 2kB
- uses mirroring
- 4 blocks of 2kB

# Connected to PPU bus

## Pattern memory (game tiles, sprites)

- 8kB (0x0000 - 0x1FFF)

## Nametables

- 0x2000 - 0x2FFF
- stores 2-dim arrays with IDs of patterns to show

## Palettes

- 0x3F00 - 0x3FFF
- 2 bits per pixel index into palette

## OAM

- location of sprites
- transfer of data from CPU to OAM with DMA (direct memory access)

# Connected to both buses

## Mapper(Cartridge)

- responsible for bank switching (cpu configures mapper to switch banks/data ranges on the cartridge)
- cpu/ppu still address the same range on cartridge but mapper points to different data/banks
- program & char rom 0x4020 - 0xFFFF
- mapper sorts out where to operate:
  - pattern memory (CHR)
  - program memory (PRG)
- bank switching via mapper:
  - mapper takes PPU and CPU wishes and translates into correct memory address
