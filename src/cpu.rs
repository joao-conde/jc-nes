use super::bus::{Bus, Device};

pub struct CPU<'a> {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u16,
    sr: u8,
    cycles_left: u8,
    bus: &'a Bus<'a>,
    flags: Flags,
}

impl<'a> CPU<'a> {
    pub fn new(bus: &'a Bus<'a>) -> CPU<'a> {
        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            pc: 0xC000,
            sp: 0xFD,
            sr: 0x00,
            cycles_left: 0,
            bus: bus,
            flags: Flags::default(),
        }
    }

    pub fn clock(&mut self) {
        if self.cycles_left == 0 {
            let opcode = self.read(self.pc);
            self.process(opcode);
        }
        self.cycles_left -= 1;
    }

    pub fn terminated(&self) -> bool {
        self.pc >= 0xFFFF
    }

    fn read(&self, address: u16) -> u8 {
        self.bus
            .read(address)
            .expect(&format!("no byte to be read at address 0x{:0x}", address))
    }

    fn process(&mut self, opcode: u8) {
        println!(
            "0x{:0x} 0x{:0x} A:0x{:0x} X:0x{:0x} Y:0x{:0x} SP:0x{:0x} CYC:{}",
            self.pc, opcode, self.a, self.x, self.y, self.sp, self.cycles_left
        );
        match opcode {
            0x00 => {
                self.brk();
                self.cycles_left += 7;
            }
            0x01 => {
                let operand = self.indx();
                self.ora(operand);
                self.cycles_left += 6;
            }
            0x18 => {
                // clc implied 1 2
                self.clc();
                self.cycles_left += 2;
            }
            0x20 => {
                let operand = self.abs();
                self.jsr(operand);
                self.cycles_left += 6;
            }
            0x38 => {
                self.sec();
                self.cycles_left += 2;
            }
            0x4C => {
                let operand = self.abs();
                self.jmp(operand);
                self.cycles_left += 3;
            }
            0x4E => {
                self.abs();
                self.lsr();
                self.cycles_left += 6;
            }
            0x86 => {
                let operand = self.zp();
                self.stx(operand);
                self.cycles_left += 3;
            }
            0xA2 => {
                let operand = self.imm();
                self.ldx(operand);
                self.cycles_left += 2;
            }
            0xA9 => {
                let operand = self.imm();
                self.lda(operand);
                self.cycles_left += 2;
            }
            0xB0 => {
                let operand = self.relative();
                println!("relative jump is 0x{:0x}", operand);
                self.bcs(operand);
                self.cycles_left += 2;
            }
            0xEA => {
                println!("---NOP---");
                self.nop();
                self.cycles_left += 2;
            }
            _ => panic!(format!(
                "invalid opcode 0x{:0x} at 0x{:0x}",
                opcode, self.pc
            )),
        }
    }

    // addressing modes
    fn abs(&mut self) -> u16 {
        self.pc += 1;
        let lo = self.read(self.pc);
        self.pc += 1;
        let hi = self.read(self.pc);
        ((hi as u16) << 8) | lo as u16
    }

    fn imm(&mut self) -> u8 {
        self.pc += 1;
        self.read(self.pc)
    }

    fn indx(&mut self) -> u8 {
        self.pc += 1;
        let address = self.read(self.pc) as u16;
        let hi = self.read((address + self.x as u16) & 0x00FF);
        let lo = self.read((address + 1 + self.x as u16) & 0x00FF);
        let address = ((hi as u16) << 8) | lo as u16;
        self.read(address)
    }

    fn relative(&mut self) -> i8 {
        self.pc += 1;
        self.read(self.pc) as i8
    }

    fn zp(&mut self) -> u16 {
        self.pc += 1;
        let lo = self.read(self.pc);
        0x0000 | lo as u16
    }

    // opcodes
    fn brk(&mut self) {
        self.flags.break_cmd = 1;
        self.pc += 1;
    }

    fn bcs(&mut self, operand: i8) {
        match self.flags.carry {
            1 => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            _ => self.pc += 1,
        }
    }

    fn clc(&mut self) {
        self.flags.carry = 0;
        self.pc += 1;
    }

    fn jmp(&mut self, operand: u16) {
        self.pc = operand;
    }

    fn jsr(&mut self, operand: u16) {
        let return_address = self.pc - 1;
        let pcl = (return_address & 0xFF) as u8;
        let pch = (return_address >> 8) as u8;
        self.write(self.sp, pcl);
        self.sp -= 1;
        self.write(self.sp, pch);
        self.sp -= 1;
        self.pc = operand;
    }

    fn lda(&mut self, operand: u8) {
        self.a = operand;
        self.flags.zero = if self.a == 0 { 1 } else { 0 };
        self.flags.negative = (self.a & 0x80) >> 7;
        self.pc += 1;
    }

    fn ldx(&mut self, operand: u8) {
        self.x = operand;
        self.flags.zero = if self.x == 0 { 1 } else { 0 };
        self.flags.negative = (self.x & 0x80) >> 7;
        self.pc += 1;
    }

    fn lsr(&mut self) {
        self.flags.carry = self.a & 0x01;
        self.a = self.a >> 1;
        self.flags.zero = if self.a == 0 { 1 } else { 0 };
        self.flags.negative = (self.a & 0x80) >> 7;
        self.a = self.a & 0x7F;
        self.pc += 1;
    }

    fn nop(&mut self) {
        self.pc += 1;
    }

    fn ora(&mut self, operand: u8) {
        self.a = self.a | operand;
        self.flags.zero = if self.a == 0 { 1 } else { 0 };
        self.flags.negative = (self.a & 0x80) >> 7;
        self.pc += 1;
    }

    fn sec(&mut self) {
        self.flags.carry = 1;
        self.pc += 1;
    }

    fn stx(&mut self, operand: u16) {
        self.write(operand, self.x);
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

#[derive(Default)]
struct Flags {
    pub carry: u8,
    pub zero: u8,
    pub interrupt: u8,
    pub break_cmd: u8,
    pub overflow: u8,
    pub negative: u8,
}
