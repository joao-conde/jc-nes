mod addressing;
mod instructions;
mod status;

use crate::bus::{Bus, Device};
use crate::cpu::status::Status;

const STACK_BASE: u16 = 0x0100;

#[derive(Default)]
pub struct Cpu {
    /// CPU registers
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    status: Status,

    /// Implementation specific
    cycle: u8,
    extra_cycles: bool,
    pub(crate) bus: Bus,
}

impl Cpu {
    pub fn new(bus: Bus) -> Cpu {
        Cpu {
            bus,
            ..Default::default()
        }
    }

    pub fn clock(&mut self) {
        if self.cycle == 0 {
            let opcode = self.bus.read(self.pc);
            self.process_opcode(opcode);
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
        self.status = Status::from(0x00);

        self.cycle = 8;
    }

    pub fn nmi(&mut self) {
        let pch = (self.pc >> 8) & 0xFF;
        let pcl = self.pc & 0x00FF;

        self.push_stack(pch as u8);
        self.push_stack(pcl as u8);

        self.status.b1 = false;
        self.status.b2 = true;
        self.status.interrupt = true;

        self.push_stack(u8::from(self.status));

        let pcl = self.bus.read(0xFFFA);
        let pch = self.bus.read(0xFFFB);
        self.pc = ((pch as u16) << 8) | pcl as u16;

        self.cycle = 8;
    }
}

/// Opcode processing and execution and utility functions
impl Cpu {
    fn process_opcode(&mut self, opcode: u8) {
        match opcode {
            // Official Opcodes
            0x00 => self.execute(Cpu::imp, Cpu::brk, 7, false),
            0x01 => self.execute(Cpu::indx, Cpu::ora, 6, false),
            0x05 => self.execute(Cpu::zp, Cpu::ora, 3, false),
            0x06 => self.execute(Cpu::zp, Cpu::asl_mem, 5, false),
            0x08 => self.execute(Cpu::imp, Cpu::php, 3, false),
            0x09 => self.execute(Cpu::imm, Cpu::ora, 2, false),
            0x0A => self.execute(Cpu::acc, Cpu::asl_acc, 2, false),
            0x0D => self.execute(Cpu::abs, Cpu::ora, 4, false),
            0x0E => self.execute(Cpu::abs, Cpu::asl_mem, 6, false),
            0x10 => self.execute(Cpu::relative, Cpu::bpl, 2, true),
            0x11 => self.execute(Cpu::indy, Cpu::ora, 5, false),
            0x15 => self.execute(Cpu::zpx, Cpu::ora, 4, false),
            0x16 => self.execute(Cpu::zpx, Cpu::asl_mem, 6, false),
            0x18 => self.execute(Cpu::imp, Cpu::clc, 2, false),
            0x19 => self.execute(Cpu::absy, Cpu::ora, 4, false),
            0x1D => self.execute(Cpu::absx, Cpu::ora, 4, false),
            0x1E => self.execute(Cpu::absx, Cpu::asl_mem, 7, false),
            0x20 => self.execute(Cpu::abs, Cpu::jsr, 6, false),
            0x21 => self.execute(Cpu::indx, Cpu::and, 6, false),
            0x24 => self.execute(Cpu::zp, Cpu::bit, 3, false),
            0x25 => self.execute(Cpu::zp, Cpu::and, 3, false),
            0x26 => self.execute(Cpu::zp, Cpu::rol_mem, 5, false),
            0x28 => self.execute(Cpu::imp, Cpu::plp, 4, false),
            0x29 => self.execute(Cpu::imm, Cpu::and, 2, false),
            0x2A => self.execute(Cpu::acc, Cpu::rol_acc, 2, false),
            0x2C => self.execute(Cpu::abs, Cpu::bit, 4, false),
            0x2D => self.execute(Cpu::abs, Cpu::and, 4, false),
            0x2E => self.execute(Cpu::abs, Cpu::rol_mem, 6, false),
            0x30 => self.execute(Cpu::relative, Cpu::bmi, 2, true),
            0x31 => self.execute(Cpu::indy, Cpu::and, 5, false),
            0x35 => self.execute(Cpu::zpx, Cpu::and, 4, false),
            0x36 => self.execute(Cpu::zpx, Cpu::rol_mem, 6, false),
            0x38 => self.execute(Cpu::imp, Cpu::sec, 2, false),
            0x39 => self.execute(Cpu::absy, Cpu::and, 4, false),
            0x3D => self.execute(Cpu::absx, Cpu::and, 4, false),
            0x3E => self.execute(Cpu::absx, Cpu::rol_mem, 7, false),
            0x40 => self.execute(Cpu::imp, Cpu::rti, 6, false),
            0x41 => self.execute(Cpu::indx, Cpu::eor, 6, false),
            0x45 => self.execute(Cpu::zp, Cpu::eor, 3, false),
            0x46 => self.execute(Cpu::zp, Cpu::lsr_mem, 5, false),
            0x48 => self.execute(Cpu::imp, Cpu::pha, 3, false),
            0x49 => self.execute(Cpu::imm, Cpu::eor, 2, false),
            0x4A => self.execute(Cpu::acc, Cpu::lsr_acc, 2, false),
            0x4C => self.execute(Cpu::abs, Cpu::jmp, 3, false),
            0x4D => self.execute(Cpu::abs, Cpu::eor, 4, false),
            0x4E => self.execute(Cpu::abs, Cpu::lsr_mem, 6, false),
            0x50 => self.execute(Cpu::relative, Cpu::bvc, 2, true),
            0x51 => self.execute(Cpu::indy, Cpu::eor, 5, false),
            0x55 => self.execute(Cpu::zpx, Cpu::eor, 4, false),
            0x56 => self.execute(Cpu::zpx, Cpu::lsr_mem, 6, false),
            0x58 => self.execute(Cpu::imp, Cpu::cli, 2, false),
            0x59 => self.execute(Cpu::absy, Cpu::eor, 4, false),
            0x5D => self.execute(Cpu::absx, Cpu::eor, 4, false),
            0x5E => self.execute(Cpu::absx, Cpu::lsr_mem, 7, false),
            0x60 => self.execute(Cpu::imp, Cpu::rts, 6, false),
            0x61 => self.execute(Cpu::indx, Cpu::adc, 6, false),
            0x65 => self.execute(Cpu::zp, Cpu::adc, 3, false),
            0x66 => self.execute(Cpu::zp, Cpu::ror_mem, 5, false),
            0x68 => self.execute(Cpu::imp, Cpu::pla, 4, false),
            0x69 => self.execute(Cpu::imm, Cpu::adc, 2, false),
            0x6A => self.execute(Cpu::acc, Cpu::ror_acc, 2, false),
            0x6C => self.execute(Cpu::ind, Cpu::jmp, 5, false),
            0x6D => self.execute(Cpu::abs, Cpu::adc, 4, false),
            0x6E => self.execute(Cpu::abs, Cpu::ror_mem, 6, false),
            0x70 => self.execute(Cpu::relative, Cpu::bvs, 2, true),
            0x71 => self.execute(Cpu::indy, Cpu::adc, 5, true),
            0x75 => self.execute(Cpu::zpx, Cpu::adc, 4, false),
            0x76 => self.execute(Cpu::zpx, Cpu::ror_mem, 6, false),
            0x78 => self.execute(Cpu::imp, Cpu::sei, 2, false),
            0x79 => self.execute(Cpu::absy, Cpu::adc, 4, true),
            0x7D => self.execute(Cpu::absx, Cpu::adc, 4, true),
            0x7E => self.execute(Cpu::absx, Cpu::ror_mem, 7, false),
            0x81 => self.execute(Cpu::indx, Cpu::sta, 6, false),
            0x84 => self.execute(Cpu::zp, Cpu::sty, 3, false),
            0x85 => self.execute(Cpu::zp, Cpu::sta, 3, false),
            0x86 => self.execute(Cpu::zp, Cpu::stx, 3, false),
            0x88 => self.execute(Cpu::imp, Cpu::dey, 2, false),
            0x8A => self.execute(Cpu::imp, Cpu::txa, 2, false),
            0x8C => self.execute(Cpu::abs, Cpu::sty, 4, false),
            0x8D => self.execute(Cpu::abs, Cpu::sta, 4, false),
            0x8E => self.execute(Cpu::abs, Cpu::stx, 4, false),
            0x90 => self.execute(Cpu::relative, Cpu::bcc, 2, true),
            0x91 => self.execute(Cpu::indy, Cpu::sta, 6, false),
            0x94 => self.execute(Cpu::zpx, Cpu::sty, 4, false),
            0x95 => self.execute(Cpu::zpx, Cpu::sta, 4, false),
            0x96 => self.execute(Cpu::zpy, Cpu::stx, 4, false),
            0x98 => self.execute(Cpu::imp, Cpu::tya, 2, false),
            0x99 => self.execute(Cpu::absy, Cpu::sta, 5, false),
            0x9A => self.execute(Cpu::imp, Cpu::txs, 2, false),
            0x9D => self.execute(Cpu::absx, Cpu::sta, 5, false),
            0xA0 => self.execute(Cpu::imm, Cpu::ldy, 2, false),
            0xA1 => self.execute(Cpu::indx, Cpu::lda, 6, false),
            0xA2 => self.execute(Cpu::imm, Cpu::ldx, 2, false),
            0xA4 => self.execute(Cpu::zp, Cpu::ldy, 3, false),
            0xA5 => self.execute(Cpu::zp, Cpu::lda, 3, false),
            0xA6 => self.execute(Cpu::zp, Cpu::ldx, 3, false),
            0xA8 => self.execute(Cpu::imp, Cpu::tay, 2, false),
            0xA9 => self.execute(Cpu::imm, Cpu::lda, 2, false),
            0xAA => self.execute(Cpu::imp, Cpu::tax, 2, false),
            0xAC => self.execute(Cpu::abs, Cpu::ldy, 4, false),
            0xAD => self.execute(Cpu::abs, Cpu::lda, 4, false),
            0xAE => self.execute(Cpu::abs, Cpu::ldx, 4, false),
            0xB0 => self.execute(Cpu::relative, Cpu::bcs, 2, true),
            0xB1 => self.execute(Cpu::indy, Cpu::lda, 5, true),
            0xB4 => self.execute(Cpu::zpx, Cpu::ldy, 4, false),
            0xB5 => self.execute(Cpu::zpx, Cpu::lda, 4, false),
            0xB6 => self.execute(Cpu::zpy, Cpu::ldx, 4, false),
            0xB8 => self.execute(Cpu::imp, Cpu::clv, 2, false),
            0xB9 => self.execute(Cpu::absy, Cpu::lda, 4, true),
            0xBA => self.execute(Cpu::imp, Cpu::tsx, 2, false),
            0xBC => self.execute(Cpu::absx, Cpu::ldy, 4, true),
            0xBD => self.execute(Cpu::absx, Cpu::lda, 4, true),
            0xBE => self.execute(Cpu::absy, Cpu::ldx, 4, true),
            0xC0 => self.execute(Cpu::imm, Cpu::cpy, 2, false),
            0xC1 => self.execute(Cpu::indx, Cpu::cmp, 6, false),
            0xC4 => self.execute(Cpu::zp, Cpu::cpy, 3, false),
            0xC5 => self.execute(Cpu::zp, Cpu::cmp, 3, false),
            0xC6 => self.execute(Cpu::zp, Cpu::dec, 5, false),
            0xC8 => self.execute(Cpu::imp, Cpu::iny, 2, false),
            0xC9 => self.execute(Cpu::imm, Cpu::cmp, 2, false),
            0xCA => self.execute(Cpu::imp, Cpu::dex, 2, false),
            0xCC => self.execute(Cpu::abs, Cpu::cpy, 4, false),
            0xCD => self.execute(Cpu::abs, Cpu::cmp, 4, false),
            0xCE => self.execute(Cpu::abs, Cpu::dec, 6, false),
            0xD0 => self.execute(Cpu::relative, Cpu::bne, 2, true),
            0xD1 => self.execute(Cpu::indy, Cpu::cmp, 5, false),
            0xD5 => self.execute(Cpu::zpx, Cpu::cmp, 4, false),
            0xD6 => self.execute(Cpu::zpx, Cpu::dec, 6, false),
            0xD8 => self.execute(Cpu::imp, Cpu::cld, 2, false),
            0xD9 => self.execute(Cpu::absy, Cpu::cmp, 4, false),
            0xDD => self.execute(Cpu::absx, Cpu::cmp, 4, false),
            0xDE => self.execute(Cpu::absx, Cpu::dec, 7, false),
            0xE0 => self.execute(Cpu::imm, Cpu::cpx, 2, false),
            0xE1 => self.execute(Cpu::indx, Cpu::sbc, 6, false),
            0xE4 => self.execute(Cpu::zp, Cpu::cpx, 3, false),
            0xE5 => self.execute(Cpu::zp, Cpu::sbc, 3, false),
            0xE6 => self.execute(Cpu::zp, Cpu::inc, 5, false),
            0xE8 => self.execute(Cpu::imp, Cpu::inx, 2, false),
            0xE9 => self.execute(Cpu::imm, Cpu::sbc, 2, false),
            0xEA => self.execute(Cpu::imp, Cpu::nop, 2, false),
            0xEC => self.execute(Cpu::abs, Cpu::cpx, 4, false),
            0xED => self.execute(Cpu::abs, Cpu::sbc, 4, false),
            0xEE => self.execute(Cpu::abs, Cpu::inc, 6, false),
            0xF0 => self.execute(Cpu::relative, Cpu::beq, 2, true),
            0xF1 => self.execute(Cpu::indy, Cpu::sbc, 5, false),
            0xF5 => self.execute(Cpu::zpx, Cpu::sbc, 4, false),
            0xF6 => self.execute(Cpu::zpx, Cpu::inc, 6, false),
            0xF8 => self.execute(Cpu::imp, Cpu::sed, 2, false),
            0xF9 => self.execute(Cpu::absy, Cpu::sbc, 4, false),
            0xFD => self.execute(Cpu::absx, Cpu::sbc, 4, false),
            0xFE => self.execute(Cpu::absx, Cpu::inc, 7, false),

            // Unofficial Opcodes (used by some ROMs)
            0x03 => self.execute(Cpu::indx, Cpu::slo, 8, false),
            0x07 => self.execute(Cpu::zp, Cpu::slo, 5, false),
            0x0F => self.execute(Cpu::abs, Cpu::slo, 6, false),
            0x13 => self.execute(Cpu::indy, Cpu::slo, 8, false),
            0x17 => self.execute(Cpu::zpx, Cpu::slo, 6, false),
            0x1B => self.execute(Cpu::absy, Cpu::slo, 7, false),
            0x1F => self.execute(Cpu::absx, Cpu::slo, 7, false),
            0x23 => self.execute(Cpu::indx, Cpu::rla, 8, false),
            0x27 => self.execute(Cpu::zp, Cpu::rla, 5, false),
            0x2F => self.execute(Cpu::abs, Cpu::rla, 6, false),
            0x33 => self.execute(Cpu::indy, Cpu::rla, 8, false),
            0x37 => self.execute(Cpu::zpx, Cpu::rla, 6, false),
            0x3B => self.execute(Cpu::absy, Cpu::rla, 7, false),
            0x3F => self.execute(Cpu::absx, Cpu::rla, 7, false),
            0x43 => self.execute(Cpu::indx, Cpu::sre, 8, false),
            0x47 => self.execute(Cpu::zp, Cpu::sre, 5, false),
            0x4F => self.execute(Cpu::abs, Cpu::sre, 6, false),
            0x53 => self.execute(Cpu::indy, Cpu::sre, 8, false),
            0x57 => self.execute(Cpu::zpx, Cpu::sre, 6, false),
            0x5B => self.execute(Cpu::absy, Cpu::sre, 7, false),
            0x5F => self.execute(Cpu::absx, Cpu::sre, 7, false),
            0x63 => self.execute(Cpu::indx, Cpu::rra, 8, false),
            0x67 => self.execute(Cpu::zp, Cpu::rra, 5, false),
            0x6F => self.execute(Cpu::abs, Cpu::rra, 6, false),
            0x73 => self.execute(Cpu::indy, Cpu::rra, 8, false),
            0x77 => self.execute(Cpu::zpx, Cpu::rra, 6, false),
            0x7B => self.execute(Cpu::absy, Cpu::rra, 7, false),
            0x7F => self.execute(Cpu::absx, Cpu::rra, 7, false),
            0x83 => self.execute(Cpu::indx, Cpu::sax, 6, false),
            0x87 => self.execute(Cpu::zp, Cpu::sax, 3, false),
            0x8F => self.execute(Cpu::abs, Cpu::sax, 4, false),
            0x97 => self.execute(Cpu::zpy, Cpu::sax, 4, false),
            0xA3 => self.execute(Cpu::indx, Cpu::lax, 6, false),
            0xA7 => self.execute(Cpu::zp, Cpu::lax, 3, false),
            0xAB => self.execute(Cpu::imm, Cpu::lax, 2, false),
            0xAF => self.execute(Cpu::abs, Cpu::lax, 4, false),
            0xB3 => self.execute(Cpu::indy, Cpu::lax, 5, true),
            0xB7 => self.execute(Cpu::zpy, Cpu::lax, 4, false),
            0xBF => self.execute(Cpu::absy, Cpu::lax, 4, true),
            0xC3 => self.execute(Cpu::indx, Cpu::dcp, 8, false),
            0xC7 => self.execute(Cpu::zp, Cpu::dcp, 5, false),
            0xCF => self.execute(Cpu::abs, Cpu::dcp, 6, false),
            0xD3 => self.execute(Cpu::indy, Cpu::dcp, 8, false),
            0xD7 => self.execute(Cpu::zpx, Cpu::dcp, 6, false),
            0xDB => self.execute(Cpu::absy, Cpu::dcp, 7, false),
            0xDF => self.execute(Cpu::absx, Cpu::dcp, 7, false),
            0xE3 => self.execute(Cpu::indx, Cpu::isc, 8, false),
            0xE7 => self.execute(Cpu::zp, Cpu::isc, 5, false),
            0xEF => self.execute(Cpu::abs, Cpu::isc, 6, false),
            0xEB => self.execute(Cpu::imm, Cpu::sbc, 2, false),
            0xF3 => self.execute(Cpu::indy, Cpu::isc, 8, false),
            0xF7 => self.execute(Cpu::zpx, Cpu::isc, 6, false),
            0xFB => self.execute(Cpu::absy, Cpu::isc, 7, false),
            0xFF => self.execute(Cpu::absx, Cpu::isc, 7, false),

            // Unofficial NOPs
            0x0C => self.execute(Cpu::abs, Cpu::nop_unoff, 4, false),
            0x04 | 0x44 | 0x64 => self.execute(Cpu::zp, Cpu::nop_unoff, 3, false),
            0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => {
                self.execute(Cpu::zpx, Cpu::nop_unoff, 4, false)
            }
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => self.execute(Cpu::imp, Cpu::nop, 2, false),
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                self.execute(Cpu::absx, Cpu::nop_unoff, 4, true)
            }
            0x80 => self.execute(Cpu::imm, Cpu::nop_unoff, 2, false),

            // Unknown Opcode
            _ => eprintln!("Unknown opcode 0x{:0X} at 0x{:0X}", opcode, self.pc),
        };
    }

    fn execute<T>(
        &mut self,
        address_mode_fn: fn(&mut Cpu) -> T,
        opcode_fn: fn(&mut Cpu, T),
        cycles: u8,
        extra_cycles: bool,
    ) {
        self.extra_cycles = extra_cycles;
        let address = address_mode_fn(self);
        opcode_fn(self, address);
        self.cycle += cycles;
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
}
