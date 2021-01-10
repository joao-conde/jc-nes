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

    pub fn terminated(&self) -> bool {
        self.tmp_total_cyc > 26554 //|| self.pc == 0xEF0A
    }
}

impl<'a> CPU<'a> {
    fn process_opcode(&mut self, opcode: u8) {
        println!(
            "{:04X} {:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            self.pc, opcode, self.a, self.x, self.y, self.status, self.sp, self.tmp_total_cyc
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

            // Unofficial opcodes
            0x03 => self.execute_instruction(CPU::indx, CPU::slo, 8, false),
            0x07 => self.execute_instruction(CPU::zp, CPU::slo, 5, false),
            0x0F => self.execute_instruction(CPU::abs, CPU::slo, 6, false),
            0x13 => self.execute_instruction(CPU::indy, CPU::slo, 8, false),
            0x17 => self.execute_instruction(CPU::zpx, CPU::slo, 6, false),
            0x1B => self.execute_instruction(CPU::absy, CPU::slo, 7, false),
            0x1F => self.execute_instruction(CPU::absx, CPU::slo, 7, false),

            0x23 => self.execute_instruction(CPU::indx, CPU::rla, 8, false),
            0x27 => self.execute_instruction(CPU::zp, CPU::rla, 5, false),
            0x2F => self.execute_instruction(CPU::abs, CPU::rla, 6, false),
            0x33 => self.execute_instruction(CPU::indy, CPU::rla, 8, false),
            0x37 => self.execute_instruction(CPU::zpx, CPU::rla, 6, false),
            0x3B => self.execute_instruction(CPU::absy, CPU::rla, 7, false),
            0x3F => self.execute_instruction(CPU::absx, CPU::rla, 7, false),

            0x43 => self.execute_instruction(CPU::indx, CPU::sre, 8, false),
            0x47 => self.execute_instruction(CPU::zp, CPU::sre, 5, false),
            0x4F => self.execute_instruction(CPU::abs, CPU::sre, 6, false),
            0x53 => self.execute_instruction(CPU::indy, CPU::sre, 8, false),
            0x57 => self.execute_instruction(CPU::zpx, CPU::sre, 6, false),
            0x5B => self.execute_instruction(CPU::absy, CPU::sre, 7, false),
            0x5F => self.execute_instruction(CPU::absx, CPU::sre, 7, false),

            0x63 => self.execute_instruction(CPU::indx, CPU::rra, 8, false),
            0x67 => self.execute_instruction(CPU::zp, CPU::rra, 5, false),
            0x6F => self.execute_instruction(CPU::abs, CPU::rra, 6, false),
            0x73 => self.execute_instruction(CPU::indy, CPU::rra, 8, false),
            0x77 => self.execute_instruction(CPU::zpx, CPU::rra, 6, false),
            0x7B => self.execute_instruction(CPU::absy, CPU::rra, 7, false),
            0x7F => self.execute_instruction(CPU::absx, CPU::rra, 7, false),

            0x83 => self.execute_instruction(CPU::indx, CPU::sax, 6, false),
            0x87 => self.execute_instruction(CPU::zp, CPU::sax, 3, false),
            0x8F => self.execute_instruction(CPU::abs, CPU::sax, 4, false),
            0x97 => self.execute_instruction(CPU::zpy, CPU::sax, 4, false),

            0xA3 => self.execute_instruction(CPU::indx, CPU::lax, 6, false),
            0xA7 => self.execute_instruction(CPU::zp, CPU::lax, 3, false),
            0xAB => self.execute_instruction(CPU::imm, CPU::lax, 2, false),
            0xAF => self.execute_instruction(CPU::abs, CPU::lax, 4, false),

            0xB3 => self.execute_instruction(CPU::indy, CPU::lax, 5, true),
            0xB7 => self.execute_instruction(CPU::zpy, CPU::lax, 4, false),
            0xBF => self.execute_instruction(CPU::absy, CPU::lax, 4, true),

            0xC3 => self.execute_instruction(CPU::indx, CPU::dcp, 8, false),
            0xC7 => self.execute_instruction(CPU::zp, CPU::dcp, 5, false),
            0xCF => self.execute_instruction(CPU::abs, CPU::dcp, 6, false),

            0xD3 => self.execute_instruction(CPU::indy, CPU::dcp, 8, false),
            0xD7 => self.execute_instruction(CPU::zpx, CPU::dcp, 6, false),
            0xDB => self.execute_instruction(CPU::absy, CPU::dcp, 7, false),
            0xDF => self.execute_instruction(CPU::absx, CPU::dcp, 7, false),

            0xE3 => self.execute_instruction(CPU::indx, CPU::isc, 8, false),
            0xE7 => self.execute_instruction(CPU::zp, CPU::isc, 5, false),
            0xEF => self.execute_instruction(CPU::abs, CPU::isc, 6, false),

            0xEB => self.execute_instruction(CPU::imm, CPU::sbc, 2, false),

            0xF3 => self.execute_instruction(CPU::indy, CPU::isc, 8, false),
            0xF7 => self.execute_instruction(CPU::zpx, CPU::isc, 6, false),
            0xFB => self.execute_instruction(CPU::absy, CPU::isc, 7, false),
            0xFF => self.execute_instruction(CPU::absx, CPU::isc, 7, false),

            // nops
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => {
                self.execute_instruction(CPU::imp, CPU::nop, 2, false)
            }
            0x80 => self.execute_instruction(CPU::imm, CPU::nop_unoff, 2, false),

            0x04 | 0x44 | 0x64 => self.execute_instruction(CPU::zp, CPU::nop_unoff, 3, false),

            0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => {
                self.execute_instruction(CPU::zpx, CPU::nop_unoff, 4, false)
            }
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                self.execute_instruction(CPU::absx, CPU::nop_unoff, 4, true)
            }
            0x0C => self.execute_instruction(CPU::abs, CPU::nop_unoff, 4, false),

            _ => panic!(format!(
                "invalid opcode 0x{:0X} at 0x{:0X}",
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

    fn is_flag_set(&self, flag: Flag) -> bool {
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

    fn relative_jump(&mut self, jump: bool, operand: i8) {
        match jump {
            true => {
                self.cycles_left += 1;
                let next = (self.pc as i32 + operand as i32) as u16 + 1;
                self.cycles_left += self.page_crossed(self.pc + 1, next) as u8;
                self.pc = next;
            }
            false => self.pc += 1,
        }
    }

    fn is_negative(&self, operand: u8) -> bool {
        (operand & 0x80) >> 7 == 1
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
        self.set_flag(Flag::Negative, self.is_negative((tmp & 0xFF) as u8));

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
        self.set_flag(Flag::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    fn asl_acc(&mut self, _acc: ()) {
        self.set_flag(Flag::Carry, self.is_negative(self.a));
        self.a <<= 1;
        self.set_flag(Flag::Negative, self.is_negative(self.a));
        self.set_flag(Flag::Zero, self.a == 0);
        self.pc += 1;
    }

    fn asl_mem(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(Flag::Carry, self.is_negative(operand));
        let operand = operand << 1;
        self.set_flag(Flag::Negative, self.is_negative(operand));
        self.set_flag(Flag::Zero, operand == 0);
        self.write(address, operand);
        self.pc += 1;
    }

    fn bcc(&mut self, address: u16) {
        self.relative_jump(!self.is_flag_set(Flag::Carry), self.read(address) as i8);
    }

    fn bcs(&mut self, address: u16) {
        self.relative_jump(self.is_flag_set(Flag::Carry), self.read(address) as i8);
    }

    fn beq(&mut self, address: u16) {
        self.relative_jump(self.is_flag_set(Flag::Zero), self.read(address) as i8);
    }

    fn bit(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(Flag::Zero, self.a & operand == 0);
        self.set_flag(Flag::Negative, self.is_negative(operand));
        self.set_flag(Flag::Overflow, (operand & 0x40) >> 6 == 1);
        self.pc += 1;
    }

    fn bmi(&mut self, address: u16) {
        self.relative_jump(self.is_flag_set(Flag::Negative), self.read(address) as i8);
    }

    fn bne(&mut self, address: u16) {
        self.relative_jump(!self.is_flag_set(Flag::Zero), self.read(address) as i8);
    }

    fn bpl(&mut self, address: u16) {
        self.relative_jump(!self.is_flag_set(Flag::Negative), self.read(address) as i8);
    }

    fn brk(&mut self, _imp: ()) {
        self.set_flag(Flag::B1, true);
        self.pc += 1;
    }

    fn bvc(&mut self, address: u16) {
        self.relative_jump(!self.is_flag_set(Flag::Overflow), self.read(address) as i8);
    }

    fn bvs(&mut self, address: u16) {
        self.relative_jump(self.is_flag_set(Flag::Overflow), self.read(address) as i8);
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
        let operand = self.read(address).wrapping_sub(1);
        self.write(address, operand);
        self.set_flag(Flag::Zero, operand == 0);
        self.set_flag(Flag::Negative, self.is_negative(operand));
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
        self.set_flag(Flag::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    fn inc(&mut self, address: u16) {
        let operand = self.read(address);
        let operand = operand.wrapping_add(1);
        self.write(address, operand);
        self.set_flag(Flag::Zero, operand == 0);
        self.set_flag(Flag::Negative, self.is_negative(operand));
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
        self.set_flag(Flag::Negative, self.is_negative(self.a));
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
        self.set_flag(Flag::Negative, self.is_negative(self.a));
        self.set_flag(Flag::Zero, self.a == 0);
        self.pc += 1;
    }

    fn lsr_mem(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(Flag::Carry, operand & 0x01 == 1);
        let operand = operand >> 1;
        self.set_flag(Flag::Negative, self.is_negative(operand));
        self.set_flag(Flag::Zero, operand == 0);
        self.write(address, operand);
        self.pc += 1;
    }

    fn nop(&mut self, _imp: ()) {
        self.pc += 1;
    }

    fn ora(&mut self, address: u16) {
        let operand = self.read(address);
        self.a |= operand;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, self.is_negative(self.a));
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
        self.set_flag(Flag::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    fn plp(&mut self, _imp: ()) {
        self.status = (self.pop_stack() & 0xEF) | 0x20; // NES quirk, not regular 6502
        self.pc += 1;
    }

    fn rol_acc(&mut self, _imp: ()) {
        let bit0 = self.is_flag_set(Flag::Carry) as u8;
        self.set_flag(Flag::Carry, self.is_negative(self.a));
        self.a <<= 1;
        self.a |= bit0;
        self.set_flag(Flag::Negative, self.is_negative(self.a));
        self.set_flag(Flag::Zero, self.a == 0);
        self.pc += 1;
    }

    fn rol_mem(&mut self, address: u16) {
        let operand = self.read(address);
        let bit0 = self.is_flag_set(Flag::Carry) as u8;
        self.set_flag(Flag::Carry, self.is_negative(operand));
        let operand = operand << 1;
        let operand = operand | bit0;
        self.write(address, operand);
        self.set_flag(Flag::Negative, self.is_negative(operand));
        self.set_flag(Flag::Zero, operand == 0);
        self.pc += 1;
    }

    fn ror_acc(&mut self, _imp: ()) {
        let bit7 = self.is_flag_set(Flag::Carry) as u8;
        self.set_flag(Flag::Carry, self.a & 0x01 == 1);
        self.a >>= 1;
        self.a |= bit7 << 7;
        self.set_flag(Flag::Negative, self.is_negative(self.a));
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
        self.set_flag(Flag::Negative, self.is_negative(operand));
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
        self.set_flag(Flag::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    fn txs(&mut self, _imp: ()) {
        self.sp = self.x;
        self.pc += 1;
    }

    fn tya(&mut self, _imp: ()) {
        self.a = self.y;
        self.set_flag(Flag::Zero, self.a == 0);
        self.set_flag(Flag::Negative, self.is_negative(self.a));
        self.pc += 1;
    }
}

/// Unofficial instructions (used by some ROMs)
impl<'a> CPU<'a> {
    fn dcp(&mut self, address: u16) {
        let operand = self.read(address).wrapping_sub(1);
        self.write(address, operand);
        self.cmp(address);
    }

    fn isc(&mut self, address: u16) {
        let operand = self.read(address).wrapping_add(1);
        self.write(address, operand);
        self.sbc(address);
    }

    fn lax(&mut self, address: u16) {
        self.lda(address);
        self.pc -= 1;
        self.ldx(address);
    }

    fn nop_unoff(&mut self, _: u16) {
        self.pc += 1;
    }

    fn rla(&mut self, address: u16) {
        self.rol_mem(address);
        self.pc -= 1;
        self.and(address);
    }

    fn sax(&mut self, address: u16) {
        self.write(address, self.a & self.x);
        self.pc += 1;
    }

    fn slo(&mut self, address: u16) {
        self.asl_mem(address);
        self.pc -= 1;
        self.ora(address);
    }

    fn sre(&mut self, address: u16) {
        self.lsr_mem(address);
        self.pc -= 1;
        self.eor(address);
    }

    fn rra(&mut self, address: u16) {
        self.ror_mem(address);
        self.pc -= 1;
        self.adc(address);
    }
}
