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
    extra_cycles: bool,
    bus: &'a mut Bus<'a>,

    tmp_total_cyc: usize, // TODO remove
}

enum Flag {
    Carry = 0,
    Zero = 1,
    Interrupt = 2,
    Decimal = 3,
    B1 = 4,
    B2 = 5,
    Overflow = 6,
    Negative = 7,
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
            tmp_total_cyc: 7,
        }
    }

    pub fn clock(&mut self) {
        if self.cycles_left == 0 {
            let opcode = self.read(self.pc);
            self.process_opcode(opcode);
        }
        self.cycles_left -= 1;
    }

    pub fn terminated(&mut self) -> bool {
        self.tmp_total_cyc > 26554 // || self.pc == 0xD924
    }
}

impl<'a> CPU<'a> {
    fn process_opcode(&mut self, opcode: u8) {
        print!(
            "{}  {}",
            format!("{:04x}", self.pc).to_uppercase(),
            to_upper_hex(opcode)
        );
        println!(
            " A:{} X:{} Y:{} P:{} SP:{} CYC:{}",
            to_upper_hex(self.a),
            to_upper_hex(self.x),
            to_upper_hex(self.y),
            to_upper_hex(self.status),
            format!("{:02x}", self.sp).to_uppercase(),
            self.tmp_total_cyc
        );
        // TODO dont forget additional clock cycles!
        match opcode {
            0x00 => self.execute_instruction(CPU::imp, CPU::brk, 7, false),
            0x01 => self.execute_instruction(CPU::indx, CPU::ora, 6, false),
            0x05 => self.execute_instruction(CPU::zp, CPU::ora, 3, false),
            0x06 => self.execute_instruction(CPU::zp, CPU::asl_mem, 5, false),
            0x08 => self.execute_instruction(CPU::imp, CPU::php, 3, false),
            0x09 => self.execute_instruction(CPU::imm, CPU::ora, 2, false),
            0x0A => self.execute_instruction(CPU::acc, CPU::asl_acc, 2, false),
            0x0D => self.execute_instruction(CPU::abs, CPU::ora, 4, false),
            0x0E => self.execute_instruction(CPU::abs, CPU::asl_mem, 6, false),
            0x10 => self.execute_instruction(CPU::relative, CPU::bpl, 2, true),
            0x11 => self.execute_instruction(CPU::indy, CPU::ora, 5, false),
            0x15 => self.execute_instruction(CPU::zpx, CPU::ora, 4, false),
            0x16 => self.execute_instruction(CPU::zpx, CPU::asl_mem, 6, false),
            0x18 => self.execute_instruction(CPU::imp, CPU::clc, 2, false),
            0x19 => self.execute_instruction(CPU::absy, CPU::ora, 4, false),
            0x1D => self.execute_instruction(CPU::absx, CPU::ora, 4, false),
            0x1E => self.execute_instruction(CPU::absx, CPU::asl_mem, 7, false),
            0x20 => self.execute_instruction(CPU::abs, CPU::jsr, 6, false),
            0x21 => self.execute_instruction(CPU::indx, CPU::and, 6, false),
            0x24 => self.execute_instruction(CPU::zp, CPU::bit, 3, false),
            0x25 => self.execute_instruction(CPU::zp, CPU::and, 3, false),
            0x26 => self.execute_instruction(CPU::zp, CPU::rol_mem, 5, false),
            0x28 => self.execute_instruction(CPU::imp, CPU::plp, 4, false),
            0x29 => self.execute_instruction(CPU::imm, CPU::and, 2, false),
            0x2A => self.execute_instruction(CPU::acc, CPU::rol_acc, 2, false),
            0x2C => self.execute_instruction(CPU::abs, CPU::bit, 4, false),
            0x2D => self.execute_instruction(CPU::abs, CPU::and, 4, false),
            0x2E => self.execute_instruction(CPU::abs, CPU::rol_mem, 6, false),
            0x30 => self.execute_instruction(CPU::relative, CPU::bmi, 2, true),
            0x31 => self.execute_instruction(CPU::indy, CPU::and, 5, false),
            0x35 => self.execute_instruction(CPU::zpx, CPU::and, 4, false),
            0x36 => self.execute_instruction(CPU::zpx, CPU::rol_mem, 6, false),
            0x38 => self.execute_instruction(CPU::imp, CPU::sec, 2, false),
            0x39 => self.execute_instruction(CPU::absy, CPU::and, 4, false),
            0x3D => self.execute_instruction(CPU::absx, CPU::and, 4, false),
            0x3E => self.execute_instruction(CPU::absx, CPU::rol_mem, 7, false),
            0x40 => self.execute_instruction(CPU::imp, CPU::rti, 6, false),
            0x41 => self.execute_instruction(CPU::indx, CPU::eor, 6, false),
            0x45 => self.execute_instruction(CPU::zp, CPU::eor, 3, false),
            0x46 => self.execute_instruction(CPU::zp, CPU::lsr_mem, 5, false),
            0x48 => self.execute_instruction(CPU::imp, CPU::pha, 3, false),
            0x49 => self.execute_instruction(CPU::imm, CPU::eor, 2, false),
            0x4A => self.execute_instruction(CPU::acc, CPU::lsr_acc, 2, false),
            0x4C => self.execute_instruction(CPU::abs, CPU::jmp, 3, false),
            0x4D => self.execute_instruction(CPU::abs, CPU::eor, 4, false),
            0x4E => self.execute_instruction(CPU::abs, CPU::lsr_mem, 6, false),
            0x50 => self.execute_instruction(CPU::relative, CPU::bvc, 2, true),
            0x51 => self.execute_instruction(CPU::indy, CPU::eor, 5, false),
            0x55 => self.execute_instruction(CPU::zpx, CPU::eor, 4, false),
            0x56 => self.execute_instruction(CPU::zpx, CPU::lsr_mem, 6, false),
            0x58 => self.execute_instruction(CPU::imp, CPU::cli, 2, false),
            0x59 => self.execute_instruction(CPU::absy, CPU::eor, 4, false),
            0x5D => self.execute_instruction(CPU::absx, CPU::eor, 4, false),
            0x5E => self.execute_instruction(CPU::absx, CPU::lsr_mem, 7, false),
            0x60 => self.execute_instruction(CPU::imp, CPU::rts, 6, false),
            0x61 => self.execute_instruction(CPU::indx, CPU::adc, 6, false),
            0x65 => self.execute_instruction(CPU::zp, CPU::adc, 3, false),
            0x66 => self.execute_instruction(CPU::zp, CPU::ror_mem, 5, false),
            0x68 => self.execute_instruction(CPU::imp, CPU::pla, 4, false),
            0x69 => self.execute_instruction(CPU::imm, CPU::adc, 2, false),
            0x6A => self.execute_instruction(CPU::acc, CPU::ror_acc, 2, false),
            0x6C => self.execute_instruction(CPU::ind, CPU::jmp, 5, false),
            0x6D => self.execute_instruction(CPU::abs, CPU::adc, 4, false),
            0x6E => self.execute_instruction(CPU::abs, CPU::ror_mem, 6, false),
            0x70 => self.execute_instruction(CPU::relative, CPU::bvs, 2, true),
            0x71 => self.execute_instruction(CPU::indy, CPU::adc, 5, true),
            0x75 => self.execute_instruction(CPU::zpx, CPU::adc, 4, false),
            0x76 => self.execute_instruction(CPU::zpx, CPU::ror_mem, 6, false),
            0x78 => self.execute_instruction(CPU::imp, CPU::sei, 2, false),
            0x79 => self.execute_instruction(CPU::absy, CPU::adc, 4, true),
            0x7D => self.execute_instruction(CPU::absx, CPU::adc, 4, true),
            0x7E => self.execute_instruction(CPU::absx, CPU::ror_mem, 7, false),
            0x81 => self.execute_instruction(CPU::indx, CPU::sta, 6, false),
            0x84 => self.execute_instruction(CPU::zp, CPU::sty, 3, false),
            0x85 => self.execute_instruction(CPU::zp, CPU::sta, 3, false),
            0x86 => self.execute_instruction(CPU::zp, CPU::stx, 3, false),
            0x88 => self.execute_instruction(CPU::imp, CPU::dey, 2, false),
            0x8A => self.execute_instruction(CPU::imp, CPU::txa, 2, false),
            0x8C => self.execute_instruction(CPU::abs, CPU::sty, 4, false),
            0x8D => self.execute_instruction(CPU::abs, CPU::sta, 4, false),
            0x8E => self.execute_instruction(CPU::abs, CPU::stx, 4, false),
            0x90 => self.execute_instruction(CPU::relative, CPU::bcc, 2, true),
            0x91 => self.execute_instruction(CPU::indy, CPU::sta, 6, false),
            0x94 => self.execute_instruction(CPU::zpx, CPU::sty, 4, false),
            0x95 => self.execute_instruction(CPU::zpx, CPU::sta, 4, false),
            0x96 => self.execute_instruction(CPU::zpy, CPU::stx, 4, false),
            0x98 => self.execute_instruction(CPU::imp, CPU::tya, 2, false),
            0x99 => self.execute_instruction(CPU::absy, CPU::sta, 5, false),
            0x9A => self.execute_instruction(CPU::imp, CPU::txs, 2, false),
            0x9D => self.execute_instruction(CPU::absx, CPU::sta, 5, false),
            0xA0 => self.execute_instruction(CPU::imm, CPU::ldy, 2, false),
            0xA1 => self.execute_instruction(CPU::indx, CPU::lda, 6, false),
            0xA2 => self.execute_instruction(CPU::imm, CPU::ldx, 2, false),
            0xA4 => self.execute_instruction(CPU::zp, CPU::ldy, 3, false),
            0xA5 => self.execute_instruction(CPU::zp, CPU::lda, 3, false),
            0xA6 => self.execute_instruction(CPU::zp, CPU::ldx, 3, false),
            0xA8 => self.execute_instruction(CPU::imp, CPU::tay, 2, false),
            0xA9 => self.execute_instruction(CPU::imm, CPU::lda, 2, false),
            0xAA => self.execute_instruction(CPU::imp, CPU::tax, 2, false),
            0xAC => self.execute_instruction(CPU::abs, CPU::ldy, 4, false),
            0xAD => self.execute_instruction(CPU::abs, CPU::lda, 4, false),
            0xAE => self.execute_instruction(CPU::abs, CPU::ldx, 4, false),
            0xB0 => self.execute_instruction(CPU::relative, CPU::bcs, 2, true),
            0xB1 => self.execute_instruction(CPU::indy, CPU::lda, 5, true),
            0xB4 => self.execute_instruction(CPU::zpx, CPU::ldy, 4, false),
            0xB5 => self.execute_instruction(CPU::zpx, CPU::lda, 4, false),
            0xB6 => self.execute_instruction(CPU::zpy, CPU::ldx, 4, false),
            0xB8 => self.execute_instruction(CPU::imp, CPU::clv, 2, false),
            0xB9 => self.execute_instruction(CPU::absy, CPU::lda, 4, true),
            0xBA => self.execute_instruction(CPU::imp, CPU::tsx, 2, false),
            0xBC => self.execute_instruction(CPU::absx, CPU::ldy, 4, true),
            0xBD => self.execute_instruction(CPU::absx, CPU::lda, 4, true),
            0xBE => self.execute_instruction(CPU::absy, CPU::ldx, 4, true),
            0xC0 => self.execute_instruction(CPU::imm, CPU::cpy, 2, false),
            0xC1 => self.execute_instruction(CPU::indx, CPU::cmp, 6, false),
            0xC4 => self.execute_instruction(CPU::zp, CPU::cpy, 3, false),
            0xC5 => self.execute_instruction(CPU::zp, CPU::cmp, 3, false),
            0xC6 => self.execute_instruction(CPU::zp, CPU::dec, 5, false),
            0xC8 => self.execute_instruction(CPU::imp, CPU::iny, 2, false),
            0xC9 => self.execute_instruction(CPU::imm, CPU::cmp, 2, false),
            0xCA => self.execute_instruction(CPU::imp, CPU::dex, 2, false),
            0xCC => self.execute_instruction(CPU::abs, CPU::cpy, 4, false),
            0xCD => self.execute_instruction(CPU::abs, CPU::cmp, 4, false),
            0xCE => self.execute_instruction(CPU::abs, CPU::dec, 6, false),
            0xD0 => self.execute_instruction(CPU::relative, CPU::bne, 2, true),
            0xD1 => self.execute_instruction(CPU::indy, CPU::cmp, 5, false),
            0xD5 => self.execute_instruction(CPU::zpx, CPU::cmp, 4, false),
            0xD6 => self.execute_instruction(CPU::zpx, CPU::dec, 6, false),
            0xD8 => self.execute_instruction(CPU::imp, CPU::cld, 2, false),
            0xD9 => self.execute_instruction(CPU::absy, CPU::cmp, 4, false),
            0xDD => self.execute_instruction(CPU::absx, CPU::cmp, 4, false),
            0xDE => self.execute_instruction(CPU::absx, CPU::dec, 7, false),
            0xE0 => self.execute_instruction(CPU::imm, CPU::cpx, 2, false),
            0xE1 => self.execute_instruction(CPU::indx, CPU::sbc, 6, false),
            0xE4 => self.execute_instruction(CPU::zp, CPU::cpx, 3, false),
            0xE5 => self.execute_instruction(CPU::zp, CPU::sbc, 3, false),
            0xE6 => self.execute_instruction(CPU::zp, CPU::inc, 5, false),
            0xE8 => self.execute_instruction(CPU::imp, CPU::inx, 2, false),
            0xE9 => self.execute_instruction(CPU::imm, CPU::sbc, 2, false),
            0xEC => self.execute_instruction(CPU::abs, CPU::cpx, 4, false),
            0xED => self.execute_instruction(CPU::abs, CPU::sbc, 4, false),
            0xEE => self.execute_instruction(CPU::abs, CPU::inc, 6, false),
            0xF0 => self.execute_instruction(CPU::relative, CPU::beq, 2, true),
            0xF1 => self.execute_instruction(CPU::indy, CPU::sbc, 5, false),
            0xF5 => self.execute_instruction(CPU::zpx, CPU::sbc, 4, false),
            0xF6 => self.execute_instruction(CPU::zpx, CPU::inc, 6, false),
            0xF8 => self.execute_instruction(CPU::imp, CPU::sed, 2, false),
            0xF9 => self.execute_instruction(CPU::absy, CPU::sbc, 4, false),
            0xFD => self.execute_instruction(CPU::absx, CPU::sbc, 4, false),
            0xFE => self.execute_instruction(CPU::absx, CPU::inc, 7, false),

            // NOPs (and illegal opcodes, false)
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => {
                self.execute_instruction(|_| 1, CPU::nop, 2, false)
            }

            0x80 | 0xEB => self.execute_instruction(|_| 2, CPU::nop, 2, false),
            0x04 | 0x44 | 0x64 | 0x87 | 0xA7 => self.execute_instruction(|_| 2, CPU::nop, 3, false),
            0x14 | 0x34 | 0x54 | 0x74 | 0x97 | 0xB7 | 0xBF | 0xD4 | 0xF4 => {
                self.execute_instruction(|_| 2, CPU::nop, 4, false)
            }
            0x07 | 0x27 | 0x47 | 0x67 | 0xC7 | 0xE7 => {
                self.execute_instruction(|_| 2, CPU::nop, 5, false)
            }
            0x17 | 0x37 | 0x57 | 0x77 | 0x83 | 0xA3 | 0xB3 | 0xD7 | 0xF7 => {
                self.execute_instruction(|_| 2, CPU::nop, 6, false)
            }
            0x03 | 0x13 | 0x23 | 0x33 | 0x43 | 0x53 | 0x63 | 0x73 | 0xC3 | 0xD3 | 0xE3 | 0xF3 => {
                self.execute_instruction(|_| 2, CPU::nop, 8, false)
            }

            0x1C | 0x0C | 0x8F | 0xAF => self.execute_instruction(|_| 3, CPU::nop, 4, false),
            0x3C | 0x5C | 0x7C | 0xDC | 0xFC => self.execute_instruction(|_| 3, CPU::nop, 5, false),
            0x0F | 0x2F | 0x4F | 0x6F | 0xCF | 0xEF => {
                self.execute_instruction(|_| 3, CPU::nop, 6, false)
            }
            0x1B | 0x1F | 0x3B | 0x3F | 0x5B | 0x5F | 0x7B | 0x7F | 0xDB | 0xDF | 0xFB | 0xFF => {
                self.execute_instruction(|_| 3, CPU::nop, 7, false)
            }

            _ => panic!(format!(
                "invalid opcode 0x{:0x} at 0x{:0x}",
                opcode, self.pc
            )),
        }
    }

    fn execute_instruction<T>(
        &mut self,
        address_mode_fn: fn(&mut CPU<'a>) -> T,
        opcode_fn: fn(&mut CPU<'a>, T),
        cycles: u8,
        extra_cycles: bool,
    ) {
        let tmp = self.cycles_left;

        self.extra_cycles = extra_cycles;
        let address = address_mode_fn(self);
        opcode_fn(self, address);
        self.cycles_left += cycles;

        self.tmp_total_cyc += (self.cycles_left - tmp) as usize;
    }

    fn push_stack(&mut self, val: u8) {
        self.write(0x0100 + self.sp as u16, val);
        self.sp -= 1;
    }

    fn pop_stack(&mut self) -> u8 {
        self.sp += 1;
        self.read(0x0100 + self.sp as u16)
    }

    fn is_flag_set(&mut self, flag: Flag) -> bool {
        (self.status >> flag as u8) & 1 == 1
    }

    fn set_flag(&mut self, flag: Flag, set_condition: bool) {
        match set_condition {
            true => self.status |= 1 << flag as u8,
            false => self.status &= !(1 << flag as u8),
        }
    }

    fn page_crossed(&self, addr1: u16, addr2: u16) -> bool {
        (addr1 & 0xFF00) != (addr2 & 0xFF00)
    }

    fn relative_jump(&mut self, operand: i8) {
        let next = (self.pc as i32 + operand as i32) as u16 + 1;
        self.cycles_left += if self.page_crossed(self.pc + 1, next) {
            2
        } else {
            1
        };
        self.pc = next;
    }
}

/// Addressing Modes
impl<'a> CPU<'a> {
    fn abs(&mut self) -> u16 {
        self.pc += 1;
        let lo = self.read(self.pc);
        self.pc += 1;
        let hi = self.read(self.pc);
        ((hi as u16) << 8) | lo as u16
    }

    fn absx(&mut self) -> u16 {
        // let address = self.abs() + self.x as u16;
        // self.cycles_left += (self.extra_cycles && self.page_crossed(self.pc + 1, address)) as u8;
        // address
        let address = self.abs();
        let hi = address & 0xFF00;
        let address = address.wrapping_add(self.x as u16);
        self.cycles_left += (self.extra_cycles && self.page_crossed(hi, address)) as u8;
        address
    }

    fn absy(&mut self) -> u16 {
        let address = self.abs();
        let hi = address & 0xFF00;
        let address = address.wrapping_add(self.y as u16);
        self.cycles_left += (self.extra_cycles && self.page_crossed(hi, address)) as u8;
        address
    }

    fn acc(&mut self) {}

    fn imm(&mut self) -> u16 {
        self.pc += 1;
        self.pc
    }

    fn imp(&mut self) {}

    fn ind(&mut self) -> u16 {
        self.pc += 1;
        let lo = self.read(self.pc);
        self.pc += 1;
        let hi = self.read(self.pc);
        let address = ((hi as u16) << 8) | lo as u16;

        // "ind" is bugged in the original hardware
        // if the low byte is 0xFF then the high byte should be read from the next page
        // the bug is that it does not, and instead just wraps around in the same page
        if lo == 0xFF {
            ((self.read(address & 0xFF00) as u16) << 8) | self.read(address) as u16
        } else {
            ((self.read(address + 1) as u16) << 8) | self.read(address) as u16
        }
    }

    fn indx(&mut self) -> u16 {
        self.pc += 1;
        let address = self.read(self.pc) as u16;
        let lo = self.read((address + self.x as u16) & 0x00FF);
        let hi = self.read((address + 1 + self.x as u16) & 0x00FF);
        ((hi as u16) << 8) + lo as u16
    }

    fn indy(&mut self) -> u16 {
        self.pc += 1;
        let address = self.read(self.pc) as u16;
        let lo = self.read(address & 0x00FF) as u16;
        let hi = self.read((address + 1) & 0x00FF);
        let hi = (hi as u16) << 8;
        let address = lo.wrapping_add(hi).wrapping_add(self.y as u16);
        self.cycles_left += (self.extra_cycles && self.page_crossed(address, hi)) as u8;
        address
    }

    fn relative(&mut self) -> u16 {
        self.pc += 1;
        self.pc
    }

    fn zp(&mut self) -> u16 {
        self.pc += 1;
        self.read(self.pc) as u16
    }

    fn zpx(&mut self) -> u16 {
        (self.zp() + self.x as u16) & 0xFF
    }

    fn zpy(&mut self) -> u16 {
        (self.zp() + self.y as u16) & 0xFF
    }
}

/// Instructions
impl<'a> CPU<'a> {
    fn adc(&mut self, address: u16) {
        let operand = self.read(address);
        let tmp = self.a as u16 + operand as u16 + self.is_flag_set(Flag::Carry) as u16;
        self.set_flag(Flag::Carry, tmp > 0xFF);
        self.set_flag(Flag::Zero, tmp & 0xFF == 0);
        self.set_flag(Flag::Negative, (tmp & 0x80) >> 7 == 1);

        // overflows if positive + positive = negative or
        // negative + negative = positive
        // V = ~(A ^ OPERAND) & (A ^ TMP)
        self.set_flag(
            Flag::Overflow,
            ((!(self.a as u16 ^ operand as u16) & (self.a as u16 ^ tmp)) & 0x0080) >> 7 == 1,
        );
        self.a = tmp as u8;
        self.pc += 1;
    }

    fn and(&mut self, address: u16) {
        let operand = self.read(address);
        self.a &= operand;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn asl_acc(&mut self, _acc: ()) {
        self.set_flag(Flag::Carry, (self.a & 0x80) >> 7 == 1);
        self.a <<= 1;
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.set_flag(Flag::Zero, self.a == 0);
        self.pc += 1;
    }

    fn asl_mem(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(Flag::Carry, (operand & 0x80) >> 7 == 1);
        let operand = operand << 1;
        self.set_flag(Flag::Negative, (operand & 0x80) >> 7 == 1);
        self.set_flag(Flag::Zero, operand == 0);
        self.write(address, operand);
        self.pc += 1;
    }

    fn bcc(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(Flag::Carry) {
            false => self.relative_jump(operand),
            true => self.pc += 1,
        }
    }

    fn bcs(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(Flag::Carry) {
            true => self.relative_jump(operand),
            false => self.pc += 1,
        }
    }

    fn beq(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(Flag::Zero) {
            true => self.relative_jump(operand),
            false => self.pc += 1,
        }
    }

    fn bit(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(Flag::Zero, self.a & operand == 0);
        self.set_flag(Flag::Negative, (operand & 0x80) >> 7 == 1);
        self.set_flag(Flag::Overflow, (operand & 0x40) >> 6 == 1);
        self.pc += 1;
    }

    fn bmi(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(Flag::Negative) {
            true => self.relative_jump(operand),
            false => self.pc += 1,
        }
    }

    fn bne(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(Flag::Zero) {
            false => self.relative_jump(operand),
            true => self.pc += 1,
        }
    }

    fn bpl(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(Flag::Negative) {
            false => self.relative_jump(operand),
            true => self.pc += 1,
        }
    }

    fn brk(&mut self, _imp: ()) {
        self.set_flag(Flag::B1, true);
        self.pc += 1;
    }

    fn bvc(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(Flag::Overflow) {
            false => self.relative_jump(operand),
            true => self.pc += 1,
        }
    }

    fn bvs(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(Flag::Overflow) {
            true => self.relative_jump(operand),
            false => self.pc += 1,
        }
    }

    fn clc(&mut self, _imp: ()) {
        self.set_flag(Flag::Carry, false);
        self.pc += 1;
    }

    fn cld(&mut self, _imp: ()) {
        self.set_flag(Flag::Decimal, false);
        self.pc += 1;
    }

    fn cli(&mut self, _imp: ()) {
        self.set_flag(Flag::Interrupt, false);
        self.pc += 1;
    }

    fn clv(&mut self, _imp: ()) {
        self.set_flag(Flag::Overflow, false);
        self.pc += 1;
    }

    fn cmp(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(Flag::Carry, self.a >= operand);
        self.set_flag(Flag::Zero, self.a == operand);
        self.set_flag(
            Flag::Negative,
            (self.a.wrapping_sub(operand) & 0x80) >> 7 == 1,
        );
        self.pc += 1;
    }

    fn cpx(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(Flag::Carry, self.x >= operand);
        self.set_flag(Flag::Zero, self.x == operand);
        self.set_flag(
            Flag::Negative,
            (self.x.wrapping_sub(operand) & 0x80) >> 7 == 1,
        );
        self.pc += 1;
    }

    fn cpy(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(Flag::Carry, self.y >= operand);
        self.set_flag(Flag::Zero, self.y == operand);
        self.set_flag(
            Flag::Negative,
            (self.y.wrapping_sub(operand) & 0x80) >> 7 == 1,
        );
        self.pc += 1;
    }

    fn dec(&mut self, address: u16) {
        let operand = self.read(address);
        let operand = operand.wrapping_sub(1);
        self.write(address, operand);
        self.set_flag(Flag::Zero, operand == 0);
        self.set_flag(Flag::Negative, (operand & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn dex(&mut self, _imp: ()) {
        self.x = self.x.wrapping_sub(1);
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn dey(&mut self, _imp: ()) {
        self.y = self.y.wrapping_sub(1);
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn eor(&mut self, address: u16) {
        let operand = self.read(address);
        self.a ^= operand;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn inc(&mut self, address: u16) {
        let operand = self.read(address);
        let operand = operand.wrapping_add(1);
        self.write(address, operand);
        self.set_flag(Flag::Zero, operand == 0);
        self.set_flag(Flag::Negative, (operand & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn inx(&mut self, _imp: ()) {
        self.x = self.x.wrapping_add(1);
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn iny(&mut self, _imp: ()) {
        self.y = self.y.wrapping_add(1);
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn jmp(&mut self, address: u16) {
        self.pc = address;
    }

    fn jsr(&mut self, address: u16) {
        let pcl = (self.pc & 0xFF) as u8;
        let pch = (self.pc >> 8) as u8;
        self.push_stack(pch);
        self.push_stack(pcl);
        self.pc = address;
    }

    fn lda(&mut self, address: u16) {
        let operand = self.read(address);
        self.a = operand;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn ldx(&mut self, address: u16) {
        let operand = self.read(address);
        self.x = operand;
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn ldy(&mut self, address: u16) {
        let operand = self.read(address);
        self.y = operand;
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn lsr_acc(&mut self, _acc: ()) {
        self.set_flag(Flag::Carry, self.a & 0x01 == 1);
        self.a >>= 1;
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.set_flag(Flag::Zero, self.a == 0);
        self.pc += 1;
    }

    fn lsr_mem(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(Flag::Carry, operand & 0x01 == 1);
        let operand = operand >> 1;
        self.set_flag(Flag::Negative, (operand & 0x80) >> 7 == 1);
        self.set_flag(Flag::Zero, operand == 0);
        self.write(address, operand);
        self.pc += 1;
    }

    fn nop(&mut self, size: u16) {
        self.pc += size;
    }

    fn ora(&mut self, address: u16) {
        let operand = self.read(address);
        self.a |= operand;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn pha(&mut self, _imp: ()) {
        self.push_stack(self.a);
        self.pc += 1;
    }

    fn php(&mut self, _imp: ()) {
        self.push_stack(self.status | 0x30); // NES quirk, not regular 6502
        self.pc += 1;
    }

    fn pla(&mut self, _imp: ()) {
        self.a = self.pop_stack();
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn plp(&mut self, _imp: ()) {
        self.status = (self.pop_stack() & 0xEF) | 0x20; // NES quirk, not regular 6502
        self.pc += 1;
    }

    fn rol_acc(&mut self, _imp: ()) {
        let bit0 = self.is_flag_set(Flag::Carry) as u8;
        self.set_flag(Flag::Carry, (self.a & 0x80) >> 7 == 1);
        self.a <<= 1;
        self.a |= bit0;
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.set_flag(Flag::Zero, self.a == 0);
        self.pc += 1;
    }

    fn rol_mem(&mut self, address: u16) {
        let operand = self.read(address);
        let bit0 = self.is_flag_set(Flag::Carry) as u8;
        self.set_flag(Flag::Carry, (operand & 0x80) >> 7 == 1);
        let operand = operand << 1;
        let operand = operand | bit0;
        self.write(address, operand);
        self.set_flag(Flag::Negative, (operand & 0x80) >> 7 == 1);
        self.set_flag(Flag::Zero, operand == 0);
        self.pc += 1;
    }

    fn ror_acc(&mut self, _imp: ()) {
        let bit7 = self.is_flag_set(Flag::Carry) as u8;
        self.set_flag(Flag::Carry, self.a & 0x01 == 1);
        self.a >>= 1;
        self.a |= bit7 << 7;
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.set_flag(Flag::Zero, self.a == 0);
        self.pc += 1;
    }

    fn ror_mem(&mut self, address: u16) {
        let operand = self.read(address);
        let bit7 = self.is_flag_set(Flag::Carry) as u8;
        self.set_flag(Flag::Carry, operand & 0x01 == 1);
        let operand = operand >> 1;
        let operand = operand | bit7 << 7;
        self.write(address, operand);
        self.set_flag(Flag::Negative, (operand & 0x80) >> 7 == 1);
        self.set_flag(Flag::Zero, operand == 0);
        self.pc += 1;
    }

    fn rti(&mut self, _imp: ()) {
        self.status = self.pop_stack();
        let pcl = self.pop_stack();
        let pch = self.pop_stack();
        self.set_flag(Flag::B2, true);
        self.pc = ((pch as u16) << 8) | pcl as u16;
    }

    fn rts(&mut self, _imp: ()) {
        let pcl = self.pop_stack();
        let pch = self.pop_stack();
        self.pc = ((pch as u16) << 8) | pcl as u16;
        self.pc += 1;
    }

    fn sbc(&mut self, address: u16) {
        let operand = self.read(address) ^ 0xFF; // 2's complement (+1 nulified by 1-C)

        // rest is the same as adc
        let tmp = self.a as u16 + operand as u16 + self.is_flag_set(Flag::Carry) as u16;
        self.set_flag(Flag::Carry, tmp > 0xFF);
        self.set_flag(Flag::Zero, tmp & 0xFF == 0);
        self.set_flag(Flag::Negative, (tmp & 0x80) >> 7 == 1);

        // overflows if positive + positive = negative or
        // negative + negative = positive
        // V = ~(A ^ OPERAND) & (A ^ TMP)
        self.set_flag(
            Flag::Overflow,
            ((!(self.a as u16 ^ operand as u16) & (self.a as u16 ^ tmp)) & 0x0080) >> 7 == 1,
        );
        self.a = tmp as u8;
        self.pc += 1;
    }

    fn sec(&mut self, _imp: ()) {
        self.set_flag(Flag::Carry, true);
        self.pc += 1;
    }

    fn sed(&mut self, _imp: ()) {
        self.set_flag(Flag::Decimal, true);
        self.pc += 1;
    }

    fn sei(&mut self, _imp: ()) {
        self.set_flag(Flag::Interrupt, true);
        self.pc += 1;
    }

    fn sta(&mut self, address: u16) {
        self.write(address, self.a);
        self.pc += 1;
    }

    fn stx(&mut self, address: u16) {
        self.write(address, self.x);
        self.pc += 1;
    }

    fn sty(&mut self, address: u16) {
        self.write(address, self.y);
        self.pc += 1;
    }

    fn tax(&mut self, _imp: ()) {
        self.x = self.a;
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn tay(&mut self, _imp: ()) {
        self.y = self.a;
        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn tsx(&mut self, _imp: ()) {
        self.x = self.sp as u8;
        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn txa(&mut self, _imp: ()) {
        self.a = self.x;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn txs(&mut self, _imp: ()) {
        self.sp = self.x;
        self.pc += 1;
    }

    fn tya(&mut self, _imp: ()) {
        self.a = self.y;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }
}

//TODO remove
fn to_upper_hex(byte: u8) -> String {
    format!("{:02x}", byte).to_uppercase()
}
