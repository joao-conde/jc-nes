use super::bus::{Bus, Device};

pub struct CPU<'a> {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u16,
    cycles_left: u8,
    bus: Bus<'a>,
    flags: Flags,
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

impl<'a> CPU<'a> {
    pub fn new(bus: Bus<'a>) -> CPU<'a> {
        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            pc: 0xC000,
            sp: 0xFD,
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

    fn push_stack(&mut self, val: u8) {
        self.write(self.sp, val);
        self.sp -= 1;
    }

    fn pop_stack(&mut self) -> u8 {
        self.sp += 1;
        let data = self.read(self.sp);
        data
    }
    
    fn process(&mut self, opcode: u8) {
        println!(
            "0x{:0x} 0x{:0x} A:0x{:0x} X:0x{:0x} Y:0x{:0x} SP:0x{:0x} CYC:{}",
            self.pc, opcode, self.a, self.x, self.y, self.sp, self.cycles_left
        );
        // TODO dont forget additional clock cycles!
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
            0x10 => {
                let operand = self.relative();
                self.bpl(operand);
                self.cycles_left += 2;
            }
            0x18 => {
                self.clc();
                self.cycles_left += 2;
            }
            0x20 => {
                let operand = self.abs();
                self.jsr(operand);
                self.cycles_left += 6;
            }
            0x24 => {
                let operand = self.zp();
                self.bit(operand);
                self.cycles_left += 3;
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
            0x50 => {
                let operand = self.relative();
                self.bvc(operand);
                self.cycles_left += 2;
            }
            0x60 => {
                self.rts();
                self.cycles_left += 6;
            }
            0x70 => {
                let operand = self.relative();
                self.bvs(operand);
                self.cycles_left += 2;
            }
            0x85 => {
                let operand = self.zp();
                self.sta(operand);
                self.cycles_left += 3;
            }
            0x86 => {
                let operand = self.zp();
                self.stx(operand);
                self.cycles_left += 3;
            }
            0x90 => {
                let operand = self.relative();
                self.bcc(operand);
                self.cycles_left += 2;
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
                self.bcs(operand);
                self.cycles_left += 2;
            }
            0xD0 => {
                let operand = self.relative();
                self.bne(operand);
                self.cycles_left += 2;
            }
            0xF0 => {
                let operand = self.relative();
                self.beq(operand);
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
}

// opcodes
impl<'a> CPU<'a> {
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

    fn bcc(&mut self, operand: i8) {
        match self.flags.carry {
            0 => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            _ => self.pc += 1,
        }
    }

    fn beq(&mut self, operand: i8) {
        match self.flags.zero {
            1 => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            _ => self.pc += 1,
        }
    }

    fn bit(&mut self, operand: u16) {
        let data = self.read(operand);
        self.flags.zero = if self.a & data == 0 { 1 } else { 0 };
        self.flags.negative = data & 0x80;
        self.flags.overflow = data & 0x40;
        self.pc += 1;
    }

    fn bne(&mut self, operand: i8) {
        match self.flags.zero {
            0 => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            _ => self.pc += 1,
        }
    }

    fn bpl(&mut self, operand: i8) {
        match self.flags.negative {
            0 => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            _ => self.pc += 1,
        }
    }

    fn bvs(&mut self, operand: i8) {
        match self.flags.overflow {
            1 => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            _ => self.pc += 1,
        }
    }

    fn bvc(&mut self, operand: i8) {
        match self.flags.overflow {
            0 => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
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
        let pcl = (self.pc & 0xFF) as u8;
        let pch = (self.pc >> 8) as u8;
        self.push_stack(pcl);
        self.push_stack(pch);
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

    fn rts(&mut self) {
        let pch = self.pop_stack();
        let pcl = self.pop_stack(); 
        self.pc = ((pch as u16) << 8) | pcl as u16;
        self.pc += 1;
    }
    
    fn sec(&mut self) {
        self.flags.carry = 1;
        self.pc += 1;
    }

    fn sta(&mut self, operand: u16) {
        self.write(operand, self.a);
        self.pc += 1;
    }

    fn stx(&mut self, operand: u16) {
        self.write(operand, self.x);
        self.pc += 1;
    }
}

// addressing modes
impl<'a> CPU<'a> {
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
}

impl<'a> Device for CPU<'a> {
    fn read(&self, address: u16) -> u8 {
        self.bus
            .read(address)
            .expect(&format!("no byte to be read at address 0x{:0x}", address))
    }

    fn write(&mut self, address: u16, data: u8) {
        self.bus.write(address, data);
    }
}
