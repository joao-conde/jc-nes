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
            pc: 0x0000,
            sp: 0x00,
            sr: 0x00,
            cycles_left: 0,
            bus: bus,
        }
    }

    pub fn clock(&mut self) {
        if self.cycles_left == 0 {
            let opcode = self.read(self.pc);
            self.process(opcode);
        }
        self.cycles_left -= 1;
    }

    fn read(&self, address: u16) -> u8 {
        self.bus
            .read(address)
            .expect(&format!("no byte to be read at address 0x{:0x}", address))
    }

    fn process(&mut self, opcode: u8) {
        println!("executing opcode 0x{:0x}", opcode);
        match opcode {
            0x00 => {
                //brk implied 1 7
            }
            0x01 => {
                let data = self.idx_ind();
                self.ora(data);
                self.cycles_left += 6;
            }
            0x4e => {
                self.abs();
                self.lsr();
                self.cycles_left += 6;
            }
            0xA9 => {
                let data = self.imm();
                self.lda(data);
                self.cycles_left += 2;
            }
            _ => {
                println!("invalid opcode '0x{:0x}'", opcode);
                self.imm();
                self.nop();
                self.cycles_left += 2;
            }
        }
    }

    // addressing modes
    fn imm(&mut self) -> u8 {
        self.pc += 1;
        self.read(self.pc)
    }

    fn abs(&mut self) -> u8 {
        self.pc += 1;
        let hi = self.read(self.pc);
        self.pc += 1;
        let lo = self.read(self.pc);
        let address = ((hi as u16) << 8) | lo as u16;
        self.read(address)
    }

    fn idx_ind(&mut self) -> u8 {
        self.pc += 1;
        let address = self.read(self.pc) as u16;
        let hi = self.read((address + self.x as u16) & 0x00FF);
        let lo = self.read((address + 1 + self.x as u16) & 0x00FF);
        let address = ((hi as u16) << 8) | lo as u16;
        self.read(address)
    }

    // opcodes
    fn lda(&mut self, val: u8) {
        self.a = val;
        self.pc += 1;
    }

    fn lsr(&mut self) {
        self.a = self.a >> 1;
        self.pc += 1;
    }

    fn nop(&mut self) {
        self.pc += 1;
    }

    fn ora(&mut self, val: u8) {
        self.a = self.a | val;
        self.pc += 1;
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
