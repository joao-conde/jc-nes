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
            let instruction = self.bus.read(self.pc).expect("no byte to read at pc");
            // let instruction = Instruction::from(instruction);

            let (size, duration) = match instruction {
                0xA9 => {
                    let data = self.bus.read(self.pc + 1).expect("no byte read at pc + 1");
                    let data = self.imm(data);
                    self.lda(data);
                    (2, 2)
                }
                _ => unreachable!(),
            };

            self.cycles_left = duration;
            self.pc += size;
        }
        self.cycles_left -= 1;
    }

    // addressing modes
    pub fn imm(&self, literal: u8) -> u8 {
        literal
    }

    // opcodes
    pub fn lda(&mut self, val: u8) {
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
