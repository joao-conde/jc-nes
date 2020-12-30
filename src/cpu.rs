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
            pc: 0xC000,
            sp: 0x00,
            sr: 0x00,
            cycles_left: 0,
            bus: bus,
        }
    }

    pub fn clock(&mut self) {
        if self.cycles_left == 0 {
            let opcode = self.bus.read(self.pc).expect("no byte to read at pc");
            self.process(opcode);
        }
        self.cycles_left -= 1;
    }

    fn process(&mut self, opcode: u8) {
        match opcode {
            0x00 => {
                println!("BRK");
                self.pc += 1;
                self.cycles_left = 7;
            }
            0xA9 => {
                let data = self
                    .bus
                    .read(self.pc + 1)
                    .expect("no byte to read at pc + 1");
                self.lda(data);
                println!("{} {} {}", self.pc, self.bus.read(self.pc).unwrap(), data);
                self.pc += 2;
                self.cycles_left = 2;
            }
            _ => panic!("invalid opcode '{}'", opcode),
        }
    }

    // addressing modes
    // fn imm(&self, literal: u8) -> u8 { // pointless, we are just emulating
    //     literal
    // }

    // opcodes
    fn lda(&mut self, val: u8) {
        self.a = val
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
