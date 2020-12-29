use super::bus::{Bus, Device};
pub struct CPU<'a> {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    sr: u8,
    cycles_left: u8,
    bus: &'a Bus<'a>,
}

impl<'a> CPU<'a> {
    pub fn new(bus: &'a Bus<'a>) -> CPU<'a> {
        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            pc: 0x00,
            sp: 0x00,
            sr: 0x00,
            cycles_left: 0,
            bus: bus
        }
    }

    pub fn clock(&mut self) {
        if self.cycles_left == 0 {
            let opcode = self.bus.read(self.pc);
            self.pc += 1;

        }
        self.cycles_left -= 1;
    }
}

impl<'a> Device for CPU<'a> {
    fn read(&self, address: u16) -> u8 {
        println!("CPU reading from {:0x}!", address);
        self.a
    }

    fn write(&mut self, address: u16, data: u8) {
        println!("CPU writing val {:0x} to {:0x}", data, address)
    }
}

struct Instruction {
    cycles: u8,
    address_mode_fn: String,
    opcode_fn: String
}