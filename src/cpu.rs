use std::ops::Add;

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
            let opcode = self.bus.read(self.pc);
            self.pc += 1;
        }
        self.cycles_left -= 1;
    }

    pub fn instruction_from_byte(&self, byte: u8) -> Instruction {
        match byte {
            0xA9 => Instruction::new(CPU::lda, CPU::imm, 2, 2),
            _ => unreachable!(),
        }
    }

    // addressing modes
    pub fn imm(cpu: &CPU, literal: u8) -> u8 {
        literal
    }

    // opcodes
    pub fn lda(cpu: &CPU, val: u8) {}
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

type AddressModeFn = fn(&CPU, u8) -> u8;
type OpcodeFn = fn(&CPU, u8);

pub struct Instruction {
    opcode_fn: OpcodeFn,
    address_mode_fn: AddressModeFn,
    size: u8,
    duration: u8,
}

impl Instruction {
    pub fn new(
        opcode_fn: OpcodeFn,
        address_mode_fn: AddressModeFn,
        duration: u8,
        size: u8,
    ) -> Instruction {
        Instruction {
            opcode_fn,
            address_mode_fn,
            size,
            duration,
        }
    }
}
