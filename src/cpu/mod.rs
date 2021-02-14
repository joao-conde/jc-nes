mod addressing;
mod instructions;

use crate::bus::Bus;
use bitflags::bitflags;

const STACK_BASE: u16 = 0x0100;

#[derive(Default)]
pub struct CPU<'a> {
    /// CPU registers
    a: u8,
    x: u8,
    y: u8,
    pub pc: u16,
    sp: u8,
    status: Status,

    /// Implementation specific
    cycle: u8,
    pub total_cycles: usize, // TODO remove ?
    extra_cycles: bool,
    pub(in crate) bus: Bus<'a>,
}

bitflags! {
    #[derive(Default)]
    pub(in crate::cpu) struct Status: u8 {
        const CARRY = 0x01;
        const ZERO = 0x02;
        const INTERRUPT = 0x04;
        const DECIMAL = 0x08;
        const B1 = 0x10;
        const B2 = 0x20;
        const OVERFLOW = 0x40;
        const NEGATIVE = 0x80;
    }
}

impl<'a> CPU<'a> {
    pub fn new(bus: Bus<'a>) -> CPU<'a> {
        let mut cpu = CPU::default();
        cpu.bus = bus;
        // nestest.nes
        // cpu.pc = 0xC000;
        // cpu.status = Status::from_bits_truncate(0x24);
        // cpu.total_cycles = 7;
        // cpu.sp = 0xFD;
        // cpu.cycle = 0;
        cpu
    }

    pub fn clock(&mut self) {
        if self.cycle == 0 {
            self.process_opcode(self.bus.read(self.pc));
        }
        self.cycle -= 1;
    }

    pub fn reset(&mut self) {
        let pcl = self.bus.read(0xFFFC);
        let pch = self.bus.read(0xFFFD);
        self.pc = ((pch as u16) << 8) | pcl as u16;

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = Status::from_bits_truncate(0x00);

        self.cycle = 8;
    }

    pub fn nmi(&mut self) {
        let pch = (self.pc >> 8) & 0xFF;
        let pcl = self.pc & 0x00FF;

        self.push_stack(pch as u8);
        self.push_stack(pcl as u8);

        self.status.set(Status::B1, false);
        self.status.set(Status::B2, true);
        self.status.set(Status::INTERRUPT, true);

        self.push_stack(self.status.bits());

        let pcl = self.bus.read(0xFFFA);
        let pch = self.bus.read(0xFFFB);
        self.pc = ((pch as u16) << 8) | pcl as u16;

        self.cycle = 8;
    }
}

/// Opcode processing and execution and utility functions
impl<'a> CPU<'a> {
    fn process_opcode(&mut self, opcode: u8) {
        match opcode {
            // Official Opcodes
            0x00 => self.execute(CPU::imp, CPU::brk, 7, false),
            0x01 => self.execute(CPU::indx, CPU::ora, 6, false),
            0x05 => self.execute(CPU::zp, CPU::ora, 3, false),
            0x06 => self.execute(CPU::zp, CPU::asl_mem, 5, false),
            0x08 => self.execute(CPU::imp, CPU::php, 3, false),
            0x09 => self.execute(CPU::imm, CPU::ora, 2, false),
            0x0A => self.execute(CPU::acc, CPU::asl_acc, 2, false),
            0x0D => self.execute(CPU::abs, CPU::ora, 4, false),
            0x0E => self.execute(CPU::abs, CPU::asl_mem, 6, false),
            0x10 => self.execute(CPU::relative, CPU::bpl, 2, true),
            0x11 => self.execute(CPU::indy, CPU::ora, 5, false),
            0x15 => self.execute(CPU::zpx, CPU::ora, 4, false),
            0x16 => self.execute(CPU::zpx, CPU::asl_mem, 6, false),
            0x18 => self.execute(CPU::imp, CPU::clc, 2, false),
            0x19 => self.execute(CPU::absy, CPU::ora, 4, false),
            0x1D => self.execute(CPU::absx, CPU::ora, 4, false),
            0x1E => self.execute(CPU::absx, CPU::asl_mem, 7, false),
            0x20 => self.execute(CPU::abs, CPU::jsr, 6, false),
            0x21 => self.execute(CPU::indx, CPU::and, 6, false),
            0x24 => self.execute(CPU::zp, CPU::bit, 3, false),
            0x25 => self.execute(CPU::zp, CPU::and, 3, false),
            0x26 => self.execute(CPU::zp, CPU::rol_mem, 5, false),
            0x28 => self.execute(CPU::imp, CPU::plp, 4, false),
            0x29 => self.execute(CPU::imm, CPU::and, 2, false),
            0x2A => self.execute(CPU::acc, CPU::rol_acc, 2, false),
            0x2C => self.execute(CPU::abs, CPU::bit, 4, false),
            0x2D => self.execute(CPU::abs, CPU::and, 4, false),
            0x2E => self.execute(CPU::abs, CPU::rol_mem, 6, false),
            0x30 => self.execute(CPU::relative, CPU::bmi, 2, true),
            0x31 => self.execute(CPU::indy, CPU::and, 5, false),
            0x35 => self.execute(CPU::zpx, CPU::and, 4, false),
            0x36 => self.execute(CPU::zpx, CPU::rol_mem, 6, false),
            0x38 => self.execute(CPU::imp, CPU::sec, 2, false),
            0x39 => self.execute(CPU::absy, CPU::and, 4, false),
            0x3D => self.execute(CPU::absx, CPU::and, 4, false),
            0x3E => self.execute(CPU::absx, CPU::rol_mem, 7, false),
            0x40 => self.execute(CPU::imp, CPU::rti, 6, false),
            0x41 => self.execute(CPU::indx, CPU::eor, 6, false),
            0x45 => self.execute(CPU::zp, CPU::eor, 3, false),
            0x46 => self.execute(CPU::zp, CPU::lsr_mem, 5, false),
            0x48 => self.execute(CPU::imp, CPU::pha, 3, false),
            0x49 => self.execute(CPU::imm, CPU::eor, 2, false),
            0x4A => self.execute(CPU::acc, CPU::lsr_acc, 2, false),
            0x4C => self.execute(CPU::abs, CPU::jmp, 3, false),
            0x4D => self.execute(CPU::abs, CPU::eor, 4, false),
            0x4E => self.execute(CPU::abs, CPU::lsr_mem, 6, false),
            0x50 => self.execute(CPU::relative, CPU::bvc, 2, true),
            0x51 => self.execute(CPU::indy, CPU::eor, 5, false),
            0x55 => self.execute(CPU::zpx, CPU::eor, 4, false),
            0x56 => self.execute(CPU::zpx, CPU::lsr_mem, 6, false),
            0x58 => self.execute(CPU::imp, CPU::cli, 2, false),
            0x59 => self.execute(CPU::absy, CPU::eor, 4, false),
            0x5D => self.execute(CPU::absx, CPU::eor, 4, false),
            0x5E => self.execute(CPU::absx, CPU::lsr_mem, 7, false),
            0x60 => self.execute(CPU::imp, CPU::rts, 6, false),
            0x61 => self.execute(CPU::indx, CPU::adc, 6, false),
            0x65 => self.execute(CPU::zp, CPU::adc, 3, false),
            0x66 => self.execute(CPU::zp, CPU::ror_mem, 5, false),
            0x68 => self.execute(CPU::imp, CPU::pla, 4, false),
            0x69 => self.execute(CPU::imm, CPU::adc, 2, false),
            0x6A => self.execute(CPU::acc, CPU::ror_acc, 2, false),
            0x6C => self.execute(CPU::ind, CPU::jmp, 5, false),
            0x6D => self.execute(CPU::abs, CPU::adc, 4, false),
            0x6E => self.execute(CPU::abs, CPU::ror_mem, 6, false),
            0x70 => self.execute(CPU::relative, CPU::bvs, 2, true),
            0x71 => self.execute(CPU::indy, CPU::adc, 5, true),
            0x75 => self.execute(CPU::zpx, CPU::adc, 4, false),
            0x76 => self.execute(CPU::zpx, CPU::ror_mem, 6, false),
            0x78 => self.execute(CPU::imp, CPU::sei, 2, false),
            0x79 => self.execute(CPU::absy, CPU::adc, 4, true),
            0x7D => self.execute(CPU::absx, CPU::adc, 4, true),
            0x7E => self.execute(CPU::absx, CPU::ror_mem, 7, false),
            0x81 => self.execute(CPU::indx, CPU::sta, 6, false),
            0x84 => self.execute(CPU::zp, CPU::sty, 3, false),
            0x85 => self.execute(CPU::zp, CPU::sta, 3, false),
            0x86 => self.execute(CPU::zp, CPU::stx, 3, false),
            0x88 => self.execute(CPU::imp, CPU::dey, 2, false),
            0x8A => self.execute(CPU::imp, CPU::txa, 2, false),
            0x8C => self.execute(CPU::abs, CPU::sty, 4, false),
            0x8D => self.execute(CPU::abs, CPU::sta, 4, false),
            0x8E => self.execute(CPU::abs, CPU::stx, 4, false),
            0x90 => self.execute(CPU::relative, CPU::bcc, 2, true),
            0x91 => self.execute(CPU::indy, CPU::sta, 6, false),
            0x94 => self.execute(CPU::zpx, CPU::sty, 4, false),
            0x95 => self.execute(CPU::zpx, CPU::sta, 4, false),
            0x96 => self.execute(CPU::zpy, CPU::stx, 4, false),
            0x98 => self.execute(CPU::imp, CPU::tya, 2, false),
            0x99 => self.execute(CPU::absy, CPU::sta, 5, false),
            0x9A => self.execute(CPU::imp, CPU::txs, 2, false),
            0x9D => self.execute(CPU::absx, CPU::sta, 5, false),
            0xA0 => self.execute(CPU::imm, CPU::ldy, 2, false),
            0xA1 => self.execute(CPU::indx, CPU::lda, 6, false),
            0xA2 => self.execute(CPU::imm, CPU::ldx, 2, false),
            0xA4 => self.execute(CPU::zp, CPU::ldy, 3, false),
            0xA5 => self.execute(CPU::zp, CPU::lda, 3, false),
            0xA6 => self.execute(CPU::zp, CPU::ldx, 3, false),
            0xA8 => self.execute(CPU::imp, CPU::tay, 2, false),
            0xA9 => self.execute(CPU::imm, CPU::lda, 2, false),
            0xAA => self.execute(CPU::imp, CPU::tax, 2, false),
            0xAC => self.execute(CPU::abs, CPU::ldy, 4, false),
            0xAD => self.execute(CPU::abs, CPU::lda, 4, false),
            0xAE => self.execute(CPU::abs, CPU::ldx, 4, false),
            0xB0 => self.execute(CPU::relative, CPU::bcs, 2, true),
            0xB1 => self.execute(CPU::indy, CPU::lda, 5, true),
            0xB4 => self.execute(CPU::zpx, CPU::ldy, 4, false),
            0xB5 => self.execute(CPU::zpx, CPU::lda, 4, false),
            0xB6 => self.execute(CPU::zpy, CPU::ldx, 4, false),
            0xB8 => self.execute(CPU::imp, CPU::clv, 2, false),
            0xB9 => self.execute(CPU::absy, CPU::lda, 4, true),
            0xBA => self.execute(CPU::imp, CPU::tsx, 2, false),
            0xBC => self.execute(CPU::absx, CPU::ldy, 4, true),
            0xBD => self.execute(CPU::absx, CPU::lda, 4, true),
            0xBE => self.execute(CPU::absy, CPU::ldx, 4, true),
            0xC0 => self.execute(CPU::imm, CPU::cpy, 2, false),
            0xC1 => self.execute(CPU::indx, CPU::cmp, 6, false),
            0xC4 => self.execute(CPU::zp, CPU::cpy, 3, false),
            0xC5 => self.execute(CPU::zp, CPU::cmp, 3, false),
            0xC6 => self.execute(CPU::zp, CPU::dec, 5, false),
            0xC8 => self.execute(CPU::imp, CPU::iny, 2, false),
            0xC9 => self.execute(CPU::imm, CPU::cmp, 2, false),
            0xCA => self.execute(CPU::imp, CPU::dex, 2, false),
            0xCC => self.execute(CPU::abs, CPU::cpy, 4, false),
            0xCD => self.execute(CPU::abs, CPU::cmp, 4, false),
            0xCE => self.execute(CPU::abs, CPU::dec, 6, false),
            0xD0 => self.execute(CPU::relative, CPU::bne, 2, true),
            0xD1 => self.execute(CPU::indy, CPU::cmp, 5, false),
            0xD5 => self.execute(CPU::zpx, CPU::cmp, 4, false),
            0xD6 => self.execute(CPU::zpx, CPU::dec, 6, false),
            0xD8 => self.execute(CPU::imp, CPU::cld, 2, false),
            0xD9 => self.execute(CPU::absy, CPU::cmp, 4, false),
            0xDD => self.execute(CPU::absx, CPU::cmp, 4, false),
            0xDE => self.execute(CPU::absx, CPU::dec, 7, false),
            0xE0 => self.execute(CPU::imm, CPU::cpx, 2, false),
            0xE1 => self.execute(CPU::indx, CPU::sbc, 6, false),
            0xE4 => self.execute(CPU::zp, CPU::cpx, 3, false),
            0xE5 => self.execute(CPU::zp, CPU::sbc, 3, false),
            0xE6 => self.execute(CPU::zp, CPU::inc, 5, false),
            0xE8 => self.execute(CPU::imp, CPU::inx, 2, false),
            0xE9 => self.execute(CPU::imm, CPU::sbc, 2, false),
            0xEA => self.execute(CPU::imp, CPU::nop, 2, false),
            0xEC => self.execute(CPU::abs, CPU::cpx, 4, false),
            0xED => self.execute(CPU::abs, CPU::sbc, 4, false),
            0xEE => self.execute(CPU::abs, CPU::inc, 6, false),
            0xF0 => self.execute(CPU::relative, CPU::beq, 2, true),
            0xF1 => self.execute(CPU::indy, CPU::sbc, 5, false),
            0xF5 => self.execute(CPU::zpx, CPU::sbc, 4, false),
            0xF6 => self.execute(CPU::zpx, CPU::inc, 6, false),
            0xF8 => self.execute(CPU::imp, CPU::sed, 2, false),
            0xF9 => self.execute(CPU::absy, CPU::sbc, 4, false),
            0xFD => self.execute(CPU::absx, CPU::sbc, 4, false),
            0xFE => self.execute(CPU::absx, CPU::inc, 7, false),

            // Unofficial Opcodes (used by some ROMs)
            0x03 => self.execute(CPU::indx, CPU::slo, 8, false),
            0x07 => self.execute(CPU::zp, CPU::slo, 5, false),
            0x0F => self.execute(CPU::abs, CPU::slo, 6, false),
            0x13 => self.execute(CPU::indy, CPU::slo, 8, false),
            0x17 => self.execute(CPU::zpx, CPU::slo, 6, false),
            0x1B => self.execute(CPU::absy, CPU::slo, 7, false),
            0x1F => self.execute(CPU::absx, CPU::slo, 7, false),
            0x23 => self.execute(CPU::indx, CPU::rla, 8, false),
            0x27 => self.execute(CPU::zp, CPU::rla, 5, false),
            0x2F => self.execute(CPU::abs, CPU::rla, 6, false),
            0x33 => self.execute(CPU::indy, CPU::rla, 8, false),
            0x37 => self.execute(CPU::zpx, CPU::rla, 6, false),
            0x3B => self.execute(CPU::absy, CPU::rla, 7, false),
            0x3F => self.execute(CPU::absx, CPU::rla, 7, false),
            0x43 => self.execute(CPU::indx, CPU::sre, 8, false),
            0x47 => self.execute(CPU::zp, CPU::sre, 5, false),
            0x4F => self.execute(CPU::abs, CPU::sre, 6, false),
            0x53 => self.execute(CPU::indy, CPU::sre, 8, false),
            0x57 => self.execute(CPU::zpx, CPU::sre, 6, false),
            0x5B => self.execute(CPU::absy, CPU::sre, 7, false),
            0x5F => self.execute(CPU::absx, CPU::sre, 7, false),
            0x63 => self.execute(CPU::indx, CPU::rra, 8, false),
            0x67 => self.execute(CPU::zp, CPU::rra, 5, false),
            0x6F => self.execute(CPU::abs, CPU::rra, 6, false),
            0x73 => self.execute(CPU::indy, CPU::rra, 8, false),
            0x77 => self.execute(CPU::zpx, CPU::rra, 6, false),
            0x7B => self.execute(CPU::absy, CPU::rra, 7, false),
            0x7F => self.execute(CPU::absx, CPU::rra, 7, false),
            0x83 => self.execute(CPU::indx, CPU::sax, 6, false),
            0x87 => self.execute(CPU::zp, CPU::sax, 3, false),
            0x8F => self.execute(CPU::abs, CPU::sax, 4, false),
            0x97 => self.execute(CPU::zpy, CPU::sax, 4, false),
            0xA3 => self.execute(CPU::indx, CPU::lax, 6, false),
            0xA7 => self.execute(CPU::zp, CPU::lax, 3, false),
            0xAB => self.execute(CPU::imm, CPU::lax, 2, false),
            0xAF => self.execute(CPU::abs, CPU::lax, 4, false),
            0xB3 => self.execute(CPU::indy, CPU::lax, 5, true),
            0xB7 => self.execute(CPU::zpy, CPU::lax, 4, false),
            0xBF => self.execute(CPU::absy, CPU::lax, 4, true),
            0xC3 => self.execute(CPU::indx, CPU::dcp, 8, false),
            0xC7 => self.execute(CPU::zp, CPU::dcp, 5, false),
            0xCF => self.execute(CPU::abs, CPU::dcp, 6, false),
            0xD3 => self.execute(CPU::indy, CPU::dcp, 8, false),
            0xD7 => self.execute(CPU::zpx, CPU::dcp, 6, false),
            0xDB => self.execute(CPU::absy, CPU::dcp, 7, false),
            0xDF => self.execute(CPU::absx, CPU::dcp, 7, false),
            0xE3 => self.execute(CPU::indx, CPU::isc, 8, false),
            0xE7 => self.execute(CPU::zp, CPU::isc, 5, false),
            0xEF => self.execute(CPU::abs, CPU::isc, 6, false),
            0xEB => self.execute(CPU::imm, CPU::sbc, 2, false),
            0xF3 => self.execute(CPU::indy, CPU::isc, 8, false),
            0xF7 => self.execute(CPU::zpx, CPU::isc, 6, false),
            0xFB => self.execute(CPU::absy, CPU::isc, 7, false),
            0xFF => self.execute(CPU::absx, CPU::isc, 7, false),

            // Unofficial NOPs
            0x0C => self.execute(CPU::abs, CPU::nop_unoff, 4, false),
            0x04 | 0x44 | 0x64 => self.execute(CPU::zp, CPU::nop_unoff, 3, false),
            0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => {
                self.execute(CPU::zpx, CPU::nop_unoff, 4, false)
            }
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => self.execute(CPU::imp, CPU::nop, 2, false),
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                self.execute(CPU::absx, CPU::nop_unoff, 4, true)
            }
            0x80 => self.execute(CPU::imm, CPU::nop_unoff, 2, false),

            // Unknown Opcode
            _ => panic!(format!(
                "Unknown opcode 0x{:0X} at 0x{:0X}",
                opcode, self.pc
            )),
        };
    }

    fn execute<T>(
        &mut self,
        address_mode_fn: fn(&mut CPU<'a>) -> T,
        opcode_fn: fn(&mut CPU<'a>, T),
        cycles: u8,
        extra_cycles: bool,
    ) {
        let tmp = self.cycle; // TODO remove ?

        self.extra_cycles = extra_cycles;
        let address = address_mode_fn(self);
        opcode_fn(self, address);
        self.cycle += cycles;

        self.total_cycles += (self.cycle - tmp) as usize; // TODO remove ?
    }

    fn push_stack(&mut self, val: u8) {
        self.bus.write(STACK_BASE + self.sp as u16, val);
        self.sp -= 1;
    }

    fn pop_stack(&mut self) -> u8 {
        self.sp += 1;
        self.bus.read(STACK_BASE + self.sp as u16)
    }

    fn relative_jump(&mut self, jump: bool, operand: i8) {
        if jump {
            self.cycle += 1;
            let next = (self.pc as i32 + operand as i32) as u16 + 1;
            self.cycle += self.page_crossed(self.pc + 1, next) as u8;
            self.pc = next;
        } else {
            self.pc += 1
        }
    }

    fn page_crossed(&self, addr1: u16, addr2: u16) -> bool {
        (addr1 & 0xFF00) != (addr2 & 0xFF00)
    }

    fn is_negative(&self, operand: u8) -> bool {
        (operand & 0x80) >> 7 == 1
    }

    pub fn debug(&self, opcode: u8) {
        println!(
            "{:04X} {:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            self.pc, opcode, self.a, self.x, self.y, self.status, self.sp, self.total_cycles
        );
    }

    pub fn pause(&self) {
        use std::io::stdin;
        let mut s = String::new();
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
    }
}
