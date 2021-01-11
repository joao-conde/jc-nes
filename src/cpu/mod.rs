mod opcodes;

use super::bus::{Bus, Device};

pub struct CPU<'a> {
    /// CPU registers
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    status: u8,

    /// Implementation specific
    cycles_left: u8,
    total_cycles: usize, // TODO remove ?
    extra_cycles: bool,
    bus: &'a mut Bus<'a>,
}

impl<'a> Device for CPU<'a> {
    fn read(&self, address: u16) -> u8 {
        self.bus
            .read(address)
            .unwrap_or_else(|| panic!("no byte to be read at address 0x{:0x}", address))
    }

    fn write(&mut self, address: u16, data: u8) {
        self.bus.write(address, data);
    }
}

impl<'a> CPU<'a> {
    pub fn new(bus: &'a mut Bus<'a>) -> CPU<'a> {
        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            pc: 0xC000,
            sp: 0xFD,
            status: 0x24,
            cycles_left: 0,
            extra_cycles: true,
            bus: bus,
            total_cycles: 7,
        }
    }

    pub fn clock(&mut self) {
        if self.cycles_left == 0 {
            let opcode = self.read(self.pc);
            self.process_opcode(opcode);
        }
        self.cycles_left -= 1;
    }
}
