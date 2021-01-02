use super::bus::{Bus, Device};

pub struct CPU<'a> {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u16,
    flags: u8,
    cycles_left: u8,
    bus: Bus<'a>,
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
            flags: 0x24,
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
        self.pc >= 0xFFFF || self.pc == 0xC83A // TODO remove
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

    fn set_flag(&mut self, flag: Flag) {
        self.flags |= 1 << flag as u8
    }

    fn unset_flag(&mut self, flag: Flag) {
        self.flags &= !(1 << flag as u8)
    }

    fn is_flag_set(&mut self, flag: Flag) -> bool {
        (self.flags >> flag as u8) & 1 == 1
    }

    fn change_flag_to(&mut self, flag: Flag, to: u8) {
        match to {
            1 => self.set_flag(flag),
            0 => self.unset_flag(flag),
            _ => panic!(format!("can't set flag to {:b} (only 0 or 1)", to)),
        }
    }

    // TODO simply change to set_flag without condition, just 0 or 1
    // then i can delete setflag,unsetflag,changeflagto
    fn set_or_unset_flag(&mut self, flag: Flag, set_condition: bool) {
        match set_condition {
            true => self.set_flag(flag),
            false => self.unset_flag(flag),
        }
    }

    fn execute_opcode<T>(
        &mut self,
        address_mode_fn: fn(&mut CPU<'a>) -> T,
        opcode_fn: fn(&mut CPU<'a>, T),
        cycles: u8,
    ) {
        let address = address_mode_fn(self);
        opcode_fn(self, address);
        self.cycles_left += cycles;
    }

    fn process_opcode(&mut self, opcode: u8) {
        // println!(
        //     "{:0x} {:0x} A:{:0x} X:{:0x} Y:{:0x} P:{:0x} SP:{:0x}",
        //     self.pc, opcode, self.a, self.x, self.y, self.flags, self.sp
        // );
        println!(
            "{:0x} {:0x} A:{:0x} P:{:0x} SP:{:0x}",
            self.pc, opcode, self.a, self.flags, self.sp
        );
        // TODO dont forget additional clock cycles!
        match opcode {
            0x00 => self.execute_opcode(CPU::imp, CPU::brk, 7),
            0x01 => self.execute_opcode(CPU::indx, CPU::ora, 6),
            0x05 => self.execute_opcode(CPU::zp, CPU::ora, 3),
            0x06 => self.execute_opcode(CPU::zp, CPU::asl_mem, 5),
            0x08 => self.execute_opcode(CPU::imp, CPU::php, 3),
            0x09 => self.execute_opcode(CPU::imm, CPU::ora, 2),
            0x0A => self.execute_opcode(CPU::acc, CPU::asl_accum, 2),
            0x0D => self.execute_opcode(CPU::abs, CPU::ora, 4),
            0x0E => self.execute_opcode(CPU::abs, CPU::asl_mem, 6),
            0x10 => self.execute_opcode(CPU::relative, CPU::bpl, 2),
            0x18 => self.execute_opcode(CPU::imp, CPU::clc, 2),
            0x20 => self.execute_opcode(CPU::abs, CPU::jsr, 6),
            0x24 => self.execute_opcode(CPU::zp, CPU::bit, 3),
            0x28 => self.execute_opcode(CPU::imp, CPU::plp, 4),
            0x29 => self.execute_opcode(CPU::imm, CPU::and, 2),
            0x30 => self.execute_opcode(CPU::relative, CPU::bmi, 2),
            0x38 => self.execute_opcode(CPU::imp, CPU::sec, 2),
            0x48 => self.execute_opcode(CPU::imp, CPU::pha, 3),
            0x49 => self.execute_opcode(CPU::imm, CPU::eor, 2),
            0x4C => self.execute_opcode(CPU::abs, CPU::jmp, 3),
            0x4E => self.execute_opcode(CPU::abs, CPU::lsr, 6),
            0x50 => self.execute_opcode(CPU::relative, CPU::bvc, 2),
            0x60 => self.execute_opcode(CPU::imp, CPU::rts, 6),
            0x68 => self.execute_opcode(CPU::imp, CPU::pla, 4),
            0x69 => self.execute_opcode(CPU::imm, CPU::adc, 2),
            0x70 => self.execute_opcode(CPU::relative, CPU::bvs, 2),
            0x78 => self.execute_opcode(CPU::imp, CPU::sei, 2),
            0x85 => self.execute_opcode(CPU::zp, CPU::sta, 3),
            0x86 => self.execute_opcode(CPU::zp, CPU::stx, 3),
            0x90 => self.execute_opcode(CPU::relative, CPU::bcc, 2),
            0xA0 => self.execute_opcode(CPU::imm, CPU::ldy, 2),
            0xA2 => self.execute_opcode(CPU::imm, CPU::ldx, 2),
            0xA9 => self.execute_opcode(CPU::imm, CPU::lda, 2),
            0xB0 => self.execute_opcode(CPU::relative, CPU::bcs, 2),
            0xB8 => self.execute_opcode(CPU::imp, CPU::clv, 2),
            0xC0 => self.execute_opcode(CPU::imm, CPU::cpy, 2),
            0xC9 => self.execute_opcode(CPU::imm, CPU::cmp, 2),
            0xD0 => self.execute_opcode(CPU::relative, CPU::bne, 2),
            0xD8 => self.execute_opcode(CPU::imp, CPU::cld, 2),
            0xE0 => self.execute_opcode(CPU::imm, CPU::cpx, 2),
            0xE9 => self.execute_opcode(CPU::imm, CPU::sbc, 2),
            0xF0 => self.execute_opcode(CPU::relative, CPU::beq, 2),
            0xF8 => self.execute_opcode(CPU::imp, CPU::sed, 2),
            0xEA => self.execute_opcode(CPU::imp, CPU::nop, 2),
            _ => panic!(format!(
                "invalid opcode 0x{:0x} at 0x{:0x}",
                opcode, self.pc
            )),
        }
    }
}

// opcodes
impl<'a> CPU<'a> {
    fn adc(&mut self, address: u16) {
        let operand = self.read(address);
        let overflow = self
            .a
            .checked_add(operand)
            .and_then(|sum| sum.checked_add(self.is_flag_set(Flag::Carry) as u8));
        self.a = self
            .a
            .wrapping_add(operand)
            .wrapping_add(self.is_flag_set(Flag::Carry) as u8);

        // TODO revisit
        self.set_or_unset_flag(Flag::Zero, self.a == 0);
        self.set_or_unset_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.set_or_unset_flag(Flag::Overflow, overflow.is_some());
        self.pc += 1;
    }

    fn and(&mut self, address: u16) {
        let operand = self.read(address);
        self.a &= operand;
        self.set_or_unset_flag(Flag::Zero, self.a == 0);
        self.set_or_unset_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn asl_accum(&mut self, _acc: ()) {
        self.set_or_unset_flag(Flag::Carry, (self.a & 0x80) >> 7 == 1);
        self.a = self.a << 1 & 0xFF; 
        self.set_or_unset_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.set_or_unset_flag(Flag::Zero, self.a == 0);
    }

    fn asl_mem(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_or_unset_flag(Flag::Carry, (operand & 0x80) >> 7 == 1);
        let operand = (operand << 1) & 0xFF;
        self.set_or_unset_flag(Flag::Negative, (operand & 0x80) >> 7 == 1);
        self.set_or_unset_flag(Flag::Zero, operand == 0);
        self.write(address, operand);
    }
    
    fn brk(&mut self, _imp: ()) {
        unreachable!();
        // self.set(Flag::B1);
        // self.set(Flag::B2);
        // self.pc += 1;
    }

    fn bcs(&mut self, operand: i8) {
        match self.is_flag_set(Flag::Carry) {
            true => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            false => self.pc += 1,
        }
    }

    fn bcc(&mut self, operand: i8) {
        match self.is_flag_set(Flag::Carry) {
            false => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            true => self.pc += 1,
        }
    }

    fn beq(&mut self, operand: i8) {
        match self.is_flag_set(Flag::Zero) {
            true => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            false => self.pc += 1,
        }
    }

    fn bit(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_or_unset_flag(Flag::Zero, self.a & operand == 0);
        self.change_flag_to(Flag::Negative, (operand & 0x80) >> 7);
        self.change_flag_to(Flag::Overflow, (operand & 0x40) >> 6);
        self.pc += 1;
    }

    fn bmi(&mut self, operand: i8) {
        match self.is_flag_set(Flag::Negative) {
            false => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            true => self.pc += 1,
        }
    }

    fn bne(&mut self, operand: i8) {
        match self.is_flag_set(Flag::Zero) {
            false => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            true => self.pc += 1,
        }
    }

    fn bpl(&mut self, operand: i8) {
        match self.is_flag_set(Flag::Negative) {
            false => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            true => self.pc += 1,
        }
    }

    fn bvs(&mut self, operand: i8) {
        match self.is_flag_set(Flag::Overflow) {
            true => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            false => self.pc += 1,
        }
    }

    fn bvc(&mut self, operand: i8) {
        match self.is_flag_set(Flag::Overflow) {
            false => self.pc = (self.pc as i32 + operand as i32) as u16 + 1,
            true => self.pc += 1,
        }
    }

    fn clc(&mut self, _imp: ()) {
        self.unset_flag(Flag::Carry);
        self.pc += 1;
    }

    fn cld(&mut self, _imp: ()) {
        self.unset_flag(Flag::Decimal);
        self.pc += 1;
    }

    fn clv(&mut self, _imp: ()) {
        self.unset_flag(Flag::Overflow);
        self.pc += 1;
    }

    fn cmp(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_or_unset_flag(Flag::Carry, self.a >= operand);
        self.set_or_unset_flag(Flag::Zero, self.a == operand);
        self.set_or_unset_flag(Flag::Negative, self.a > operand);
        self.pc += 1;
    }

    fn cpx(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_or_unset_flag(Flag::Carry, self.x >= operand);
        self.set_or_unset_flag(Flag::Zero, self.x == operand);
        self.set_or_unset_flag(Flag::Negative, self.x > operand);
        self.pc += 1;
    }

    fn cpy(&mut self, address: u16) {
        let operand = self.read(address);
        self.set_or_unset_flag(Flag::Carry, self.y >= operand);
        self.set_or_unset_flag(Flag::Zero, self.y == operand);
        self.set_or_unset_flag(Flag::Negative, self.y > operand);
        self.pc += 1;
    }

    fn eor(&mut self, address: u16) {
        let operand = self.read(address);
        self.a ^= operand;
        self.set_or_unset_flag(Flag::Zero, self.a == 0);
        self.set_or_unset_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
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
        self.set_or_unset_flag(Flag::Zero, self.a == 0);
        self.set_or_unset_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn ldx(&mut self, address: u16) {
        let operand = self.read(address);
        self.x = operand;
        self.set_or_unset_flag(Flag::Zero, self.x == 0);
        self.set_or_unset_flag(Flag::Negative, self.x & 0x80 == 1);
        self.pc += 1;
    }

    fn ldy(&mut self, address: u16) {
        let operand = self.read(address);
        self.y = operand;
        self.set_or_unset_flag(Flag::Zero, self.y == 0);
        self.set_or_unset_flag(Flag::Negative, self.y & 0x80 == 1);
        self.pc += 1;
    }

    fn lsr(&mut self, operand: u16) {
        self.change_flag_to(Flag::Carry, self.a & 0x01);
        self.a = self.a >> 1;
        self.set_or_unset_flag(Flag::Zero, self.a == 0);
        self.set_or_unset_flag(Flag::Negative, self.a & 0x80 == 1);
        self.a = self.a & 0x7F;
        self.pc += 1;
    }

    fn nop(&mut self, _imp: ()) {
        println!("---NOP---");
        self.pc += 1;
    }

    fn ora(&mut self, address: u16) {
        let operand = self.read(address);
        self.a = self.a | operand;
        self.set_or_unset_flag(Flag::Zero, self.a == 0);
        self.set_or_unset_flag(Flag::Negative, self.a & 0x80 == 1);
        self.pc += 1;
    }

    fn pha(&mut self, _imp: ()) {
        self.push_stack(self.a);
        self.pc += 1;
    }

    fn php(&mut self, _imp: ()) {
        self.set_flag(Flag::B1);
        self.push_stack(self.flags | 0x30);
        self.pc += 1;
    }

    fn pla(&mut self, _imp: ()) {
        self.a = self.pop_stack();
        self.set_or_unset_flag(Flag::Zero, self.a == 0);
        self.set_or_unset_flag(Flag::Negative, (self.a & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    fn plp(&mut self, _imp: ()) {
        self.flags = self.pop_stack();
        self.set_flag(Flag::B2);
        self.pc += 1;
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
        self.set_flag(Flag::Carry);
        self.pc += 1;
    }

    fn sed(&mut self, _imp: ()) {
        self.set_flag(Flag::Decimal);
        self.pc += 1;
    }

    fn sei(&mut self, _imp: ()) {
        self.set_flag(Flag::Interrupt);
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

    fn acc(&mut self) {
        ()
    }

    fn imm(&mut self) -> u16 {
        self.pc += 1;
        self.pc
    }

    fn imp(&mut self) {
        ()
    }

    fn indx(&mut self) -> u16 {
        self.pc += 1;
        let address = self.read(self.pc) as u16;
        let hi = self.read((address + self.x as u16) & 0x00FF);
        let lo = self.read((address + 1 + self.x as u16) & 0x00FF);
        ((hi as u16) << 8) | lo as u16
    }

    fn relative(&mut self) -> i8 {
        self.pc += 1;
        self.read(self.pc) as i8
    }

    fn zp(&mut self) -> u16 {
        self.pc += 1;
        self.read(self.pc) as u16
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
