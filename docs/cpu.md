# Central Processing Unit (CPU)

## Overview

- [Memory model](http://wiki.nesdev.com/w/index.php/CPU_memory_map)
- Based on the MOS6502's processor (lacking DECIMAL mode and with some quirks)
- Runs at approximately 1.79 MHz
- 64kB addressable range (0x0000 - 0xFFFF)

## Registers

| Register        | Type  |
| --------------- | ----- |
| Accumulator     | `u8`  |
| Register X      | `u8`  |
| Register Y      | `u8`  |
| Program Counter | `u16` |
| Stack Pointer   | `u8`  |
| Status Register | `u8`  |

## Addressing Modes

- 13 [addressing modes](http://www.obelisk.me.uk/6502/addressing.html)
- Each specifies how to fetch the data for the next instruction

## Instructions

- 56 [legal instructions](http://www.obelisk.me.uk/6502/instructions.html)
- Together with the addressing mode, take different number of clock cycles to execute (duration)
- Together with the addressing mode, can be 1, 2 or 3 bytes long (size)
- First byte of the instruction specifies the addressing mode, instruction, duration and size
- Illegal instructions (that some ROMs use):
  - http://www.ffd2.com/fridge/docs/6502-NMOS.extra.opcodes
  - http://www.oxyron.de/html/opcodes02.html

## Executing The Program

1. Read the byte at PC (program counter) location
2. Find the instruction, addressing mode, duration and size by examining that byte
3. Execute the instruction and update the PC (size) and cycle count (duration)

## Development With The `nestest` ROM

Analyze current byte pointed by the PC and if you do not have it implemented yet:
1. Check page 10 of [this datasheet](http://archive.6502.org/datasheets/rockwell_r650x_r651x.pdf)
2. An opcode `0xA1` will match row `A` and column `1` (e.g. `LDA INDX 2 6`)
3. Implement the instruction and the addressing mode (e.g. `LDA` and `INDX`, respectively), updating PC by `size` (e.g. 2) and cycle count by `duration` (e.g. 6)

While you implement the addressing modes and instructions, test them by:
1. Downloading the [nestest.nes ROM](https://github.com/christopherpow/nes-test-roms/blob/master/other/nestest.nes)
2. Skip the header (first 16 bytes) save the remaining bytes in your program memory starting at `0xC000`
3. Set `PC=0xC000`
4. Execute and compare your logs with the the [nestest.nes logs](https://github.com/christopherpow/nes-test-roms/blob/master/other/nestest.log)

## NES CPU Quirks

The NES CPU is based on a 6502 yet it is not the same:

- Besides lacking `DECIMAL` mode, the behavior of the status bits 4 and 5 (from 0 to 7) is quite bizarre; check [this link](https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag) and pay close attention to the PHP/PLP instructions
- The indirect (IND) addressing mode was bugged in the original hardware; to mimic the original behavior, you have to reproduce the bug behavior in this addressing mode (it should cross page boundaries, instead it just wraps around)

## Other References

- http://wiki.nesdev.com/w/index.php/CPU
- https://skilldrick.github.io/easy6502/
- http://archive.6502.org/datasheets/rockwell_r650x_r651x.pdf
