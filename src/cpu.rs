use super::bus::{Bus, Device};

pub struct CPU<'a> {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u16,
    status: u8,
    cycles_left: u8,
    bus: Bus<'a>,

    tmp_total_cyc: usize, // TODO remove
}

enum StatusFlag {
    Carry = 0,
    Zero = 1,
    Interrupt = 2,
    Decimal = 3,
    B1 = 4,
    B2 = 5, //TODO remove
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
    pub fn new(bus: Bus<'a>) -> CPU<'a> {
        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            pc: 0xC000,
            sp: 0xFD,
            cycles_left: 0,
            bus: bus,
            status: 0x24,
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
        self.pc >= 0xFFFF || self.pc == 0xCE51 // TODO remove
    }
}

impl<'a> CPU<'a> {
    fn push_stack(&mut self, val: u8) {
        self.write(self.sp, val);
        self.sp -= 1;
    }

    fn pop_stack(&mut self) -> u8 {
        self.sp += 1;
        self.read(self.sp)
    }

    fn is_flag_set(&mut self, flag: StatusFlag) -> bool {
        (self.status >> flag as u8) & 1 == 1
    }

    fn set_flag(&mut self, flag: StatusFlag, set_condition: bool) {
        match set_condition {
            true => self.status |= 1 << flag as u8,
            false => self.status &= !(1 << flag as u8),
        }
    }

    fn execute_instruction<T>(
        &mut self,
        address_mode_fn: fn(&mut CPU<'a>) -> T,
        opcode_fn: fn(&mut CPU<'a>, T),
        cycles: u8,
    ) {
        let address = address_mode_fn(self);
        opcode_fn(self, address);
        self.cycles_left += cycles;
        self.tmp_total_cyc += cycles as usize;
    }

    fn process_opcode(&mut self, opcode: u8) {
        println!(
            "{:0x} {:0x} A:{:0x} X:{:0x} Y:{:0x} P:{:0x} SP:{:0x} CYC:{}",
            self.pc, opcode, self.a, self.x, self.y, self.status, self.sp, self.tmp_total_cyc
        );
        // println!(
        //     "{:0x} {:0x} A:{:0x} P:{:0x} SP:{:0x}",
        //     self.pc, opcode, self.a, self.flags, self.sp
        // );
        // TODO dont forget additional clock cycles!
        match opcode {
            0x00 => self.execute_instruction(CPU::imp, CPU::brk, 7),
            0x01 => self.execute_instruction(CPU::indx, CPU::ora, 6),
            0x05 => self.execute_instruction(CPU::zp, CPU::ora, 3),
            0x06 => self.execute_instruction(CPU::zp, CPU::asl_mem, 5),
            0x08 => self.execute_instruction(CPU::imp, CPU::php, 3),
            0x09 => self.execute_instruction(CPU::imm, CPU::ora, 2),
            0x0A => self.execute_instruction(CPU::acc, CPU::asl_acc, 2),
            0x0D => self.execute_instruction(CPU::abs, CPU::ora, 4),
            0x0E => self.execute_instruction(CPU::abs, CPU::asl_mem, 6),
            0x10 => self.execute_instruction(CPU::relative, CPU::bpl, 2),
            0x11 => self.execute_instruction(CPU::indy, CPU::ora, 5),
            0x15 => self.execute_instruction(CPU::zpx, CPU::ora, 4),
            0x16 => self.execute_instruction(CPU::zpx, CPU::asl_mem, 6),
            0x18 => self.execute_instruction(CPU::imp, CPU::clc, 2),
            0x19 => self.execute_instruction(CPU::absy, CPU::ora, 4),
            0x1D => self.execute_instruction(CPU::absx, CPU::ora, 4),
            0x1E => self.execute_instruction(CPU::absx, CPU::asl_mem, 7),
            0x20 => self.execute_instruction(CPU::abs, CPU::jsr, 6),
            0x21 => self.execute_instruction(CPU::indx, CPU::and, 6),
            0x24 => self.execute_instruction(CPU::zp, CPU::bit, 3),
            0x25 => self.execute_instruction(CPU::zp, CPU::and, 3),
            0x26 => unimplemented!(),
            0x28 => self.execute_instruction(CPU::imp, CPU::plp, 4),
            0x29 => self.execute_instruction(CPU::imm, CPU::and, 2),
            0x2A => unimplemented!(),
            0x2C => self.execute_instruction(CPU::abs, CPU::bit, 4),
            0x2D => self.execute_instruction(CPU::abs, CPU::and, 4),
            0x2E => unimplemented!(),
            0x30 => self.execute_instruction(CPU::relative, CPU::bmi, 2),
            0x31 => self.execute_instruction(CPU::indy, CPU::and, 5),
            0x35 => self.execute_instruction(CPU::zpx, CPU::and, 4),
            0x36 => unimplemented!(),
            0x38 => self.execute_instruction(CPU::imp, CPU::sec, 2),
            0x39 => self.execute_instruction(CPU::absy, CPU::and, 4),
            0x3D => self.execute_instruction(CPU::absx, CPU::and, 4),
            0x3E => unimplemented!(),
            0x40 => self.execute_instruction(CPU::imp, CPU::rti, 6),
            0x41 => self.execute_instruction(CPU::indx, CPU::eor, 6),
            0x45 => self.execute_instruction(CPU::zp, CPU::eor, 3),
            0x46 => self.execute_instruction(CPU::zp, CPU::lsr_mem, 5),
            0x48 => self.execute_instruction(CPU::imp, CPU::pha, 3),
            0x49 => self.execute_instruction(CPU::imm, CPU::eor, 2),
            0x4A => self.execute_instruction(CPU::acc, CPU::lsr_acc, 2),
            0x4C => self.execute_instruction(CPU::abs, CPU::jmp, 3),
            0x4D => self.execute_instruction(CPU::abs, CPU::eor, 4),
            0x4E => self.execute_instruction(CPU::abs, CPU::lsr_mem, 6),
            0x50 => self.execute_instruction(CPU::relative, CPU::bvc, 2),
            0x51 => self.execute_instruction(CPU::indy, CPU::eor, 5),
            0x55 => self.execute_instruction(CPU::zpx, CPU::eor, 4),
            0x56 => self.execute_instruction(CPU::zpx, CPU::lsr_mem, 6),
            0x58 => self.execute_instruction(CPU::imp, CPU::cli, 2),
            0x59 => self.execute_instruction(CPU::absy, CPU::eor, 4),
            0x5D => self.execute_instruction(CPU::absx, CPU::eor, 4),
            0x5E => self.execute_instruction(CPU::absx, CPU::lsr_mem, 7),
            0x60 => self.execute_instruction(CPU::imp, CPU::rts, 6),
            0x61 => self.execute_instruction(CPU::indx, CPU::adc, 6),
            0x65 => self.execute_instruction(CPU::zp, CPU::adc, 3),
            0x66 => unimplemented!(),
            0x68 => self.execute_instruction(CPU::imp, CPU::pla, 4),
            0x69 => self.execute_instruction(CPU::imm, CPU::adc, 2),
            0x6A => unimplemented!(),
            0x6C => self.execute_instruction(CPU::ind, CPU::jmp, 5),
            0x6D => self.execute_instruction(CPU::abs, CPU::adc, 4),
            0x6E => unimplemented!(),
            0x70 => self.execute_instruction(CPU::relative, CPU::bvs, 2),
            0x71 => self.execute_instruction(CPU::indy, CPU::adc, 5),
            0x75 => self.execute_instruction(CPU::zpx, CPU::adc, 4),
            0x76 => unimplemented!(),
            0x78 => self.execute_instruction(CPU::imp, CPU::sei, 2),
            0x79 => self.execute_instruction(CPU::absy, CPU::adc, 4),
            0x7D => self.execute_instruction(CPU::absx, CPU::adc, 4),
            0x7E => unimplemented!(),
            0x81 => self.execute_instruction(CPU::indx, CPU::sta, 6),
            0x84 => self.execute_instruction(CPU::zp, CPU::sty, 3),
            0x85 => self.execute_instruction(CPU::zp, CPU::sta, 3),
            0x86 => self.execute_instruction(CPU::zp, CPU::stx, 3),
            0x88 => self.execute_instruction(CPU::imp, CPU::dey, 2),
            0x8A => self.execute_instruction(CPU::imp, CPU::txa, 2),
            0x8C => self.execute_instruction(CPU::abs, CPU::sty, 4),
            0x8D => self.execute_instruction(CPU::abs, CPU::sta, 4),
            0x8E => self.execute_instruction(CPU::abs, CPU::stx, 4),
            0x90 => self.execute_instruction(CPU::relative, CPU::bcc, 2),
            0x91 => self.execute_instruction(CPU::indy, CPU::sta, 6),
            0x94 => self.execute_instruction(CPU::zpx, CPU::sty, 4),
            0x95 => self.execute_instruction(CPU::zpx, CPU::sta, 4),
            0x96 => self.execute_instruction(CPU::zpy, CPU::stx, 4),
            0x98 => self.execute_instruction(CPU::imp, CPU::tya, 2),
            0x99 => self.execute_instruction(CPU::absy, CPU::sta, 5),
            0x9A => self.execute_instruction(CPU::imp, CPU::txs, 2),
            0x9D => self.execute_instruction(CPU::absx, CPU::sta, 5),
            0xA0 => self.execute_instruction(CPU::imm, CPU::ldy, 2),
            0xA1 => self.execute_instruction(CPU::indx, CPU::lda, 6),
            0xA2 => self.execute_instruction(CPU::imm, CPU::ldx, 2),
            0xA4 => self.execute_instruction(CPU::zp, CPU::ldy, 3),
            0xA5 => self.execute_instruction(CPU::zp, CPU::lda, 3),
            0xA6 => self.execute_instruction(CPU::zp, CPU::ldx, 3),
            0xA8 => self.execute_instruction(CPU::imp, CPU::tay, 2),
            0xA9 => self.execute_instruction(CPU::imm, CPU::lda, 2),
            0xAA => self.execute_instruction(CPU::imp, CPU::tax, 2),
            0xAC => self.execute_instruction(CPU::abs, CPU::ldy, 4),
            0xAD => self.execute_instruction(CPU::abs, CPU::lda, 4),
            0xAE => self.execute_instruction(CPU::abs, CPU::ldx, 4),
            0xB0 => self.execute_instruction(CPU::relative, CPU::bcs, 2),
            0xB1 => self.execute_instruction(CPU::indy, CPU::lda, 5),
            0xB4 => self.execute_instruction(CPU::zpx, CPU::ldy, 4),
            0xB5 => self.execute_instruction(CPU::zpx, CPU::lda, 4),
            0xB6 => self.execute_instruction(CPU::zpy, CPU::ldx, 4),
            0xB8 => self.execute_instruction(CPU::imp, CPU::clv, 2),
            0xB9 => self.execute_instruction(CPU::absy, CPU::lda, 4),
            0xBA => self.execute_instruction(CPU::imp, CPU::tsx, 2),
            0xBC => self.execute_instruction(CPU::absx, CPU::ldy, 4),
            0xBD => self.execute_instruction(CPU::absx, CPU::lda, 4),
            0xBE => self.execute_instruction(CPU::absy, CPU::ldx, 4),
            0xC0 => self.execute_instruction(CPU::imm, CPU::cpy, 2),
            0xC1 => self.execute_instruction(CPU::indx, CPU::cmp, 6),
            0xC4 => self.execute_instruction(CPU::zp, CPU::cpy, 3),
            0xC5 => self.execute_instruction(CPU::zp, CPU::cmp, 3),
            0xC6 => self.execute_instruction(CPU::zp, CPU::dec, 5),
            0xC8 => self.execute_instruction(CPU::imp, CPU::iny, 2),
            0xC9 => self.execute_instruction(CPU::imm, CPU::cmp, 2),
            0xCA => self.execute_instruction(CPU::imp, CPU::dex, 2),
            0xCC => self.execute_instruction(CPU::abs, CPU::cpy, 4),
            0xCD => self.execute_instruction(CPU::abs, CPU::cmp, 4),
            0xCE => self.execute_instruction(CPU::abs, CPU::dec, 6),
            0xD0 => self.execute_instruction(CPU::relative, CPU::bne, 2),
            0xD1 => self.execute_instruction(CPU::indy, CPU::cmp, 5),
            0xD5 => self.execute_instruction(CPU::zpx, CPU::cmp, 4),
            0xD6 => self.execute_instruction(CPU::zpx, CPU::dec, 6),
            0xD8 => self.execute_instruction(CPU::imp, CPU::cld, 2),
            0xD9 => self.execute_instruction(CPU::absy, CPU::cmp, 4),
            0xDD => self.execute_instruction(CPU::absx, CPU::cmp, 4),
            0xDE => self.execute_instruction(CPU::absx, CPU::dec, 7),
            0xE0 => self.execute_instruction(CPU::imm, CPU::cpx, 2),
            0xE1 => self.execute_instruction(CPU::indx, CPU::sbc, 6),
            0xE4 => self.execute_instruction(CPU::zp, CPU::cpx, 3),
            0xE5 => self.execute_instruction(CPU::zp, CPU::sbc, 3),
            0xE6 => self.execute_instruction(CPU::zp, CPU::inc, 5),
            0xE8 => self.execute_instruction(CPU::imp, CPU::inx, 2),
            0xE9 => self.execute_instruction(CPU::imm, CPU::sbc, 2),
            0xEA => self.execute_instruction(CPU::imp, CPU::nop, 2),
            0xEC => self.execute_instruction(CPU::abs, CPU::cpx, 4),
            0xED => self.execute_instruction(CPU::abs, CPU::sbc, 4),
            0xEE => self.execute_instruction(CPU::abs, CPU::inc, 6),
            0xF0 => self.execute_instruction(CPU::relative, CPU::beq, 2),
            0xF1 => self.execute_instruction(CPU::indy, CPU::sbc, 5),
            0xF5 => self.execute_instruction(CPU::zpx, CPU::sbc, 4),
            0xF6 => self.execute_instruction(CPU::zpx, CPU::inc, 6),
            0xF8 => self.execute_instruction(CPU::imp, CPU::sed, 2),
            0xF9 => self.execute_instruction(CPU::absy, CPU::sbc, 4),
            0xFD => self.execute_instruction(CPU::absx, CPU::sbc, 4),
            0xFE => self.execute_instruction(CPU::absx, CPU::inc, 7),
            _ => panic!(format!(
                "invalid opcode 0x{:0x} at 0x{:0x}",
                opcode, self.pc
            )),
        }
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

    fn absy(&mut self) -> u16 {
        self.abs() + self.y as u16
    }

    fn absx(&mut self) -> u16 {
        self.abs() + self.x as u16
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
        ((hi as u16) << 8) + lo as u16
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
        let lo = self.read(address) as u16;
        let hi = self.read((address + 1) & 0x00FF);
        let hi = (hi as u16) << 8;
        lo + hi + self.y as u16
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
        self.zp() + self.x as u16
    }

    fn zpy(&mut self) -> u16 {
        self.zp() + self.y as u16
    }
}

/// Instructions
impl<'a> CPU<'a> {
    fn adc(&mut self, address: u16) {
        let operand = self.read(address);
        let overflow = self
            .a
            .checked_add(operand)
            .and_then(|sum| sum.checked_add(self.is_flag_set(StatusFlag::Carry) as u8));
        self.a = self
            .a
            .wrapping_add(operand)
            .wrapping_add(self.is_flag_set(StatusFlag::Carry) as u8);

        // TODO revisit
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.set_flag(StatusFlag::Overflow, overflow.is_some());
        self.pc += 1;
    }

    fn and(&mut self, address: u16) {
        let operand = self.read(address);
        self.a &= operand;
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn asl_acc(&mut self, _acc: ()) {
        self.set_flag(StatusFlag::Carry, (self.a & 0x80) >> 7 == 1);
        self.a <<= 1;
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.set_flag(StatusFlag::Zero, self.a == 0);
    }

    fn asl_mem(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(StatusFlag::Carry, (operand & 0x80) >> 7 == 1);
        let operand = operand << 1;
        self.set_flag(StatusFlag::Negative, (operand & 0x80) >> 7 == 1);
        self.set_flag(StatusFlag::Zero, operand == 0);
        self.write(address, operand);
    }

    fn brk(&mut self, _imp: ()) {
        self.set_flag(StatusFlag::B1, true);
        self.pc += 1;
    }

    fn bcs(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(StatusFlag::Carry) {
            true => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            false => self.pc += 1,
        }
    }

    fn bcc(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(StatusFlag::Carry) {
            false => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            true => self.pc += 1,
        }
    }

    fn beq(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(StatusFlag::Zero) {
            true => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            false => self.pc += 1,
        }
    }

    fn bit(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(StatusFlag::Zero, self.a & operand == 0);
        self.set_flag(StatusFlag::Negative, (operand & 0x80) >> 7 == 1);
        self.set_flag(StatusFlag::Overflow, (operand & 0x40) >> 6 == 1);
        self.pc += 1;
    }

    fn bmi(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(StatusFlag::Negative) {
            true => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            false => self.pc += 1,
        }
    }

    fn bne(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(StatusFlag::Zero) {
            false => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            true => self.pc += 1,
        }
    }

    fn bpl(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(StatusFlag::Negative) {
            false => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            true => self.pc += 1,
        }
    }

    fn bvs(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(StatusFlag::Overflow) {
            true => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            false => self.pc += 1,
        }
    }

    fn bvc(&mut self, address: u16) {
        let operand = self.read(address) as i8;
        match self.is_flag_set(StatusFlag::Overflow) {
            false => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            true => self.pc += 1,
        }
    }

    fn clc(&mut self, _imp: ()) {
        self.set_flag(StatusFlag::Carry, false);
        self.pc += 1;
    }

    fn cld(&mut self, _imp: ()) {
        self.set_flag(StatusFlag::Decimal, false);
        self.pc += 1;
    }

    fn cli(&mut self, _imp: ()) {
        self.set_flag(StatusFlag::Interrupt, false);
        self.pc += 1;
    }

    fn clv(&mut self, _imp: ()) {
        self.set_flag(StatusFlag::Overflow, false);
        self.pc += 1;
    }

    fn cmp(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(StatusFlag::Carry, self.a >= operand);
        self.set_flag(StatusFlag::Zero, self.a == operand);
        self.set_flag(StatusFlag::Negative, self.a > operand);
        self.pc += 1;
    }

    fn cpx(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(StatusFlag::Carry, self.x >= operand);
        self.set_flag(StatusFlag::Zero, self.x == operand);
        self.set_flag(StatusFlag::Negative, self.x > operand);
        self.pc += 1;
    }

    fn cpy(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(StatusFlag::Carry, self.y >= operand);
        self.set_flag(StatusFlag::Zero, self.y == operand);
        self.set_flag(StatusFlag::Negative, self.y > operand);
        self.pc += 1;
    }

    fn dec(&mut self, address: u16) {
        let operand = self.read(address);
        let operand = operand - 1;
        self.write(address, operand);
        self.set_flag(StatusFlag::Zero, operand == 0);
        self.set_flag(StatusFlag::Negative, (operand & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn dex(&mut self, _imp: ()) {
        self.x = self.x.wrapping_sub(1);
        self.set_flag(StatusFlag::Zero, self.x == 0);
        self.set_flag(StatusFlag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn dey(&mut self, _imp: ()) {
        self.y = self.y.wrapping_sub(1);
        self.set_flag(StatusFlag::Zero, self.y == 0);
        self.set_flag(StatusFlag::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn eor(&mut self, address: u16) {
        let operand = self.read(address);
        self.a ^= operand;
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn inc(&mut self, address: u16) {
        let operand = self.read(address);
        let operand = operand + 1;
        self.write(address, operand);
        self.set_flag(StatusFlag::Zero, operand == 0);
        self.set_flag(StatusFlag::Negative, (operand & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn inx(&mut self, _imp: ()) {
        self.x = self.x.wrapping_add(1);
        self.set_flag(StatusFlag::Zero, self.x == 0);
        self.set_flag(StatusFlag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn iny(&mut self, _imp: ()) {
        self.y = self.y.wrapping_add(1);
        self.set_flag(StatusFlag::Zero, self.y == 0);
        self.set_flag(StatusFlag::Negative, (self.y & 0x80) >> 7 == 1);
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

    fn lda(&mut self, address: u16) {
        let operand = self.read(address);
        self.a = operand;
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn ldx(&mut self, address: u16) {
        let operand = self.read(address);
        self.x = operand;
        self.set_flag(StatusFlag::Zero, self.x == 0);
        self.set_flag(StatusFlag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn ldy(&mut self, address: u16) {
        let operand = self.read(address);
        self.y = operand;
        self.set_flag(StatusFlag::Zero, self.y == 0);
        self.set_flag(StatusFlag::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn lsr_mem(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_flag(StatusFlag::Carry, operand & 0x01 == 1);
        let operand = operand >> 1;
        self.set_flag(StatusFlag::Zero, operand == 0);
        self.set_flag(StatusFlag::Negative, (operand & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn lsr_acc(&mut self, _acc: ()) {
        self.set_flag(StatusFlag::Carry, (self.a & 0x80) >> 7 == 1);
        self.a <<= 1;
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.set_flag(StatusFlag::Zero, self.a == 0);
    }

    fn nop(&mut self, _imp: ()) {
        println!("---NOP---");
        self.pc += 1;
    }

    fn ora(&mut self, address: u16) {
        let operand = self.read(address);
        self.a |= operand;
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn pha(&mut self, _imp: ()) {
        self.push_stack(self.a);
        self.pc += 1;
    }

    fn php(&mut self, _imp: ()) {
        self.push_stack(self.status | 0x10); // NES quirk, not regular 6502
        self.pc += 1;
    }

    fn pla(&mut self, _imp: ()) {
        self.a = self.pop_stack();
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn plp(&mut self, _imp: ()) {
        self.status = self.pop_stack() & 0xEF; // NES quirk, not regular 6502
        self.pc += 1;
    }

    fn rti(&mut self, _imp: ()) {
        self.status = self.pop_stack();
        let pch = self.pop_stack();
        let pcl = self.pop_stack();
        self.pc = ((pch as u16) << 8) | pcl as u16;
    }

    fn rts(&mut self, _imp: ()) {
        let pch = self.pop_stack();
        let pcl = self.pop_stack();
        self.pc = ((pch as u16) << 8) | pcl as u16;
        self.pc += 1;
    }

    fn sbc(&mut self, address: u16) {
        let operand = self.read(address);
        //TODO just like ADC
        self.pc += 1;
    }

    fn sec(&mut self, _imp: ()) {
        self.set_flag(StatusFlag::Carry, true);
        self.pc += 1;
    }

    fn sed(&mut self, _imp: ()) {
        self.set_flag(StatusFlag::Decimal, true);
        self.pc += 1;
    }

    fn sei(&mut self, _imp: ()) {
        self.set_flag(StatusFlag::Interrupt, true);
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

    fn sty(&mut self, operand: u16) {
        self.write(operand, self.y);
        self.pc += 1;
    }

    fn tax(&mut self, _imp: ()) {
        self.x = self.a;
        self.set_flag(StatusFlag::Zero, self.x == 0);
        self.set_flag(StatusFlag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn tay(&mut self, _imp: ()) {
        self.y = self.a;
        self.set_flag(StatusFlag::Zero, self.y == 0);
        self.set_flag(StatusFlag::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn tsx(&mut self, _imp: ()) {
        self.x = self.sp as u8;
        self.set_flag(StatusFlag::Zero, self.x == 0);
        self.set_flag(StatusFlag::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn txa(&mut self, _imp: ()) {
        self.a = self.x;
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn txs(&mut self, _imp: ()) {
        self.sp = self.x as u16;
        self.pc += 1;
    }

    fn tya(&mut self, _imp: ()) {
        self.a = self.y;
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }
}
