# Picture Processing Unit (PPU)

## Overview

- [Memory model](http://wiki.nesdev.com/w/index.php/PPU_memory_map)
- Runs three times faster than the CPU
- 16kB addressable range (0x0000 - 0x3FFF)
- Outputs pixel at each clock cycle

## Registers

All eight registers are memory-mapped and available to the CPU:

| Register    | Memory Address | Type |
| ----------- | -------------- | ---- |
| Control     | `0x2000`         | `u8` |
| Mask        | `0x2001`         | `u8` |
| Status      | `0x2002`         | `u8` |
| OAM Address | `0x2003`         | `u8` |
| OAM Data    | `0x2004`         | `u8` |
| Scroll      | `0x2005`         | `u8` |
| Address     | `0x2006`         | `u8` |
| Data        | `0x2007`         | `u8` |
| OAM DMA     | `0x4014`         | `u8` |
