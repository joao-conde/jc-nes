# [CPU](http://wiki.nesdev.com/w/index.php/CPU)

- Central Processing Unit
- based on the MOS6502 processor 
- runs at approximately 1.79 MHz 
- made by Ricoh and lacks the MOS6502's decimal mode
- 64kB addressable range (0x0000 - 0xFFFF)
- [registers](http://wiki.nesdev.com/w/index.php/CPU_registers):
    - 1 byte each
    - A: accumulator 
    - X: register
    - Y: register
    - PC: program counter
    - SP: stack pointer
    - SR: status register

- 64kB addressable range (0x0000 - 0xFFFF -> 16-bit addresses)
- can output address (16-bit address)
- can read/write data (1 byte)
- works on clock cycles


## Instructions

- 56 legal instructions
- take different number of clock cycles to execute (duration)
- can be 1, 2 or 3 bytes long (size)
- first byte of the instruction specifies duration and size
- different addressing modes
- page 10 of http://archive.6502.org/datasheets/rockwell_r650x_r651x.pdf

1. read byte @ PC location
2. opcode[byte] -> addressing mode, duration, size
3. execute
4. wait, count cycles, complete
