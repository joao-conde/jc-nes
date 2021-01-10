# CPU

http://www.obelisk.me.uk/6502/architecture.html

- based on the MOS6502's processor (lacking decimal mode and with some quirks)
- runs at approximately 1.79 MHz
- 64kB addressable range (0x0000 - 0xFFFF)
- can read/write 1 byte at a time

## Registers

http://www.obelisk.me.uk/6502/registers.html

| Register        | Size (bytes) |
| --------------- | ------------ |
| Accumulator     | 1            |
| Register X      | 1            |
| Register Y      | 1            |
| Program Counter | 2            |
| Stack Pointer   | 1            |
| Status Register | 1            |

## Addressing Modes

http://www.obelisk.me.uk/6502/addressing.html

- 13 addressing modes
- each specifies a way in which memory should be addressed

## Instructions

http://www.obelisk.me.uk/6502/instructions.html

- 56 legal instructions 
- together with the addressing mode, take different number of clock cycles to execute (duration)
- together with the addressing mode, can be 1, 2 or 3 bytes long (size)
- first byte of the instruction specifies operation, duration and size
- illegal instructions (that some ROMs use):
  - http://www.ffd2.com/fridge/docs/6502-NMOS.extra.opcodes
  - http://www.oxyron.de/html/opcodes02.html

## Executing The Program

1. read the byte at PC (program counter) location
2. find the addressing mode, duration and size of the instruction by looking at that byte
3. execute the instruction and update the PC (size) and cycle count (duration)

## Test Driven Development

Analyze current byte pointed by the PC and if you do not have it implemented yet:
1. check page 10 of http://archive.6502.org/datasheets/rockwell_r650x_r651x.pdf
2. an opcode 0xA1 will match row A and column 1 (e.g. LDA INDX 2 6)
3. implement the instruction and the addressing mode (e.g. LDA and INDX, respectively), updating PC by size (e.g. 2) and cycle count by duration (e.g. 6)

After (or while) you implement the addressing modes and instructions:
1. read the entire description of the `nestest.nes` ROM here https://github.com/christopherpow/nes-test-roms/blob/master/other/nestest.txt
2. download the `nestest.nes` ROM from here https://github.com/christopherpow/nes-test-roms/blob/master/other/nestest.nes
3. skip the header, the first 16 bits, save the remaining bytes starting at 0xC000
4. point your CPU PC at 0xC000
5. execute and compare your logs with the the `nestest.nes` logs https://github.com/christopherpow/nes-test-roms/blob/master/other/nestest.log

## NES CPU Quirks

The NES CPU is based on a 6502 yet it is not the same:

- besides lacking decimal mode, the behavior of the status bits 4 and 5 (from 0 to 7) is quite bizarre; check https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag and pay close attention to the PHP/PLP instructions
- the indirect (IND) addressing mode was bugged in the original hardware; to mimic the original behavior, you have to reproduce the bug behavior in this addressing mode (it should cross page boundaries, instead it just wraps around)

## Other References

- http://wiki.nesdev.com/w/index.php/CPU
- http://archive.6502.org/datasheets/rockwell_r650x_r651x.pdf
- https://skilldrick.github.io/easy6502/
