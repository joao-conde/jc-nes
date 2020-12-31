# CPU

- https://github.com/skilldrick/6502js
- http://archive.6502.org/datasheets/rockwell_r650x_r651x.pdf
- https://skilldrick.github.io/easy6502/
- http://wiki.nesdev.com/w/index.php/CPU
- http://www.obelisk.me.uk/6502/
- http://www.obelisk.me.uk/6502/reference.html
- http://www.6502.org/tutorials/6502opcodes.html
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
- https://gist.github.com/bit-hack/3be16a5333a50732d3eee85c962157a7

1. read byte @ PC location
2. opcode[byte] -> addressing mode, duration, size
3. execute
4. wait, count cycles, complete


Thoughts:

- Addressing modes matter to get data (where to get it from)
- Opcode tell us what to do with that data and where to store result
- So really all its needed is:
  - a mapper from first byte to addressing mode and operation to execute, as well as size and duration
  - take the next [size] bytes of data using the addressing function and feed it to the operation
  - operation stores where it needs to
  - increase pc by size
  - catch return of operation to check if additional cycles are required


## Testing

- https://github.com/christopherpow/nes-test-roms/blob/master/other/nestest.log
- https://github.com/christopherpow/nes-test-roms/blob/master/other/nestest.txt
- https://github.com/christopherpow/nes-test-roms/blob/master/other/nestest.nes