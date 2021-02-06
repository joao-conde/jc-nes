use crate::cpu::{Status, CPU};

/// Instructions
impl<'a> CPU<'a> {
    pub(in crate::cpu) fn adc(&mut self, address: u16) {
        let operand = self.bus.read(address);
        let tmp = self.a as u16 + operand as u16 + self.is_set(Status::Carry) as u16;
        self.set_status_bit(Status::Carry, tmp > 0xFF);
        self.set_status_bit(Status::Zero, tmp & 0xFF == 0);
        self.set_status_bit(Status::Negative, self.is_negative((tmp & 0xFF) as u8));

        // overflows if positive + positive = negative or
        // negative + negative = positive
        // V = ~(A ^ OPERAND) & (A ^ TMP)
        self.set_status_bit(
            Status::Overflow,
            ((!(self.a as u16 ^ operand as u16) & (self.a as u16 ^ tmp)) & 0x0080) >> 7 == 1,
        );
        self.a = tmp as u8;
        self.pc += 1;
    }

    pub(in crate::cpu) fn and(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a &= operand;
        self.set_status_bit(Status::Zero, self.a == 0);
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    pub(in crate::cpu) fn asl_acc(&mut self, _acc: ()) {
        self.set_status_bit(Status::Carry, self.is_negative(self.a));
        self.a <<= 1;
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.set_status_bit(Status::Zero, self.a == 0);
        self.pc += 1;
    }

    pub(in crate::cpu) fn asl_mem(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.set_status_bit(Status::Carry, self.is_negative(operand));
        let operand = operand << 1;
        self.set_status_bit(Status::Negative, self.is_negative(operand));
        self.set_status_bit(Status::Zero, operand == 0);
        self.bus.write(address, operand);
        self.pc += 1;
    }

    pub(in crate::cpu) fn bcc(&mut self, address: u16) {
        self.relative_jump(!self.is_set(Status::Carry), self.bus.read(address) as i8);
    }

    pub(in crate::cpu) fn bcs(&mut self, address: u16) {
        self.relative_jump(self.is_set(Status::Carry), self.bus.read(address) as i8);
    }

    pub(in crate::cpu) fn beq(&mut self, address: u16) {
        self.relative_jump(self.is_set(Status::Zero), self.bus.read(address) as i8);
    }

    pub(in crate::cpu) fn bit(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.set_status_bit(Status::Zero, self.a & operand == 0);
        self.set_status_bit(Status::Negative, self.is_negative(operand));
        self.set_status_bit(Status::Overflow, (operand & 0x40) >> 6 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn bmi(&mut self, address: u16) {
        self.relative_jump(self.is_set(Status::Negative), self.bus.read(address) as i8);
    }

    pub(in crate::cpu) fn bne(&mut self, address: u16) {
        self.relative_jump(!self.is_set(Status::Zero), self.bus.read(address) as i8);
    }

    pub(in crate::cpu) fn bpl(&mut self, address: u16) {
        self.relative_jump(!self.is_set(Status::Negative), self.bus.read(address) as i8);
    }

    pub(in crate::cpu) fn brk(&mut self, _imp: ()) {
        self.set_status_bit(Status::B1, true);
        self.pc += 1;
    }

    pub(in crate::cpu) fn bvc(&mut self, address: u16) {
        self.relative_jump(!self.is_set(Status::Overflow), self.bus.read(address) as i8);
    }

    pub(in crate::cpu) fn bvs(&mut self, address: u16) {
        self.relative_jump(self.is_set(Status::Overflow), self.bus.read(address) as i8);
    }

    pub(in crate::cpu) fn clc(&mut self, _imp: ()) {
        self.set_status_bit(Status::Carry, false);
        self.pc += 1;
    }

    pub(in crate::cpu) fn cld(&mut self, _imp: ()) {
        self.set_status_bit(Status::Decimal, false);
        self.pc += 1;
    }

    pub(in crate::cpu) fn cli(&mut self, _imp: ()) {
        self.set_status_bit(Status::Interrupt, false);
        self.pc += 1;
    }

    pub(in crate::cpu) fn clv(&mut self, _imp: ()) {
        self.set_status_bit(Status::Overflow, false);
        self.pc += 1;
    }

    pub(in crate::cpu) fn cmp(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.set_status_bit(Status::Carry, self.a >= operand);
        self.set_status_bit(Status::Zero, self.a == operand);
        self.set_status_bit(
            Status::Negative,
            (self.a.wrapping_sub(operand) & 0x80) >> 7 == 1,
        );
        self.pc += 1;
    }

    pub(in crate::cpu) fn cpx(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.set_status_bit(Status::Carry, self.x >= operand);
        self.set_status_bit(Status::Zero, self.x == operand);
        self.set_status_bit(
            Status::Negative,
            (self.x.wrapping_sub(operand) & 0x80) >> 7 == 1,
        );
        self.pc += 1;
    }

    pub(in crate::cpu) fn cpy(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.set_status_bit(Status::Carry, self.y >= operand);
        self.set_status_bit(Status::Zero, self.y == operand);
        self.set_status_bit(
            Status::Negative,
            (self.y.wrapping_sub(operand) & 0x80) >> 7 == 1,
        );
        self.pc += 1;
    }

    pub(in crate::cpu) fn dec(&mut self, address: u16) {
        let operand = self.bus.read(address).wrapping_sub(1);
        self.bus.write(address, operand);
        self.set_status_bit(Status::Zero, operand == 0);
        self.set_status_bit(Status::Negative, self.is_negative(operand));
        self.pc += 1;
    }

    pub(in crate::cpu) fn dex(&mut self, _imp: ()) {
        self.x = self.x.wrapping_sub(1);
        self.set_status_bit(Status::Zero, self.x == 0);
        self.set_status_bit(Status::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn dey(&mut self, _imp: ()) {
        self.y = self.y.wrapping_sub(1);
        self.set_status_bit(Status::Zero, self.y == 0);
        self.set_status_bit(Status::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn eor(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a ^= operand;
        self.set_status_bit(Status::Zero, self.a == 0);
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    pub(in crate::cpu) fn inc(&mut self, address: u16) {
        let operand = self.bus.read(address);
        let operand = operand.wrapping_add(1);
        self.bus.write(address, operand);
        self.set_status_bit(Status::Zero, operand == 0);
        self.set_status_bit(Status::Negative, self.is_negative(operand));
        self.pc += 1;
    }

    pub(in crate::cpu) fn inx(&mut self, _imp: ()) {
        self.x = self.x.wrapping_add(1);
        self.set_status_bit(Status::Zero, self.x == 0);
        self.set_status_bit(Status::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn iny(&mut self, _imp: ()) {
        self.y = self.y.wrapping_add(1);
        self.set_status_bit(Status::Zero, self.y == 0);
        self.set_status_bit(Status::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn jmp(&mut self, address: u16) {
        self.pc = address;
    }

    pub(in crate::cpu) fn jsr(&mut self, address: u16) {
        let pcl = (self.pc & 0xFF) as u8;
        let pch = (self.pc >> 8) as u8;
        self.push_stack(pch);
        self.push_stack(pcl);
        self.pc = address;
    }

    pub(in crate::cpu) fn lda(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a = operand;
        self.set_status_bit(Status::Zero, self.a == 0);
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    pub(in crate::cpu) fn ldx(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.x = operand;
        self.set_status_bit(Status::Zero, self.x == 0);
        self.set_status_bit(Status::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn ldy(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.y = operand;
        self.set_status_bit(Status::Zero, self.y == 0);
        self.set_status_bit(Status::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn lsr_acc(&mut self, _acc: ()) {
        self.set_status_bit(Status::Carry, self.a & 0x01 == 1);
        self.a >>= 1;
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.set_status_bit(Status::Zero, self.a == 0);
        self.pc += 1;
    }

    pub(in crate::cpu) fn lsr_mem(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.set_status_bit(Status::Carry, operand & 0x01 == 1);
        let operand = operand >> 1;
        self.set_status_bit(Status::Negative, self.is_negative(operand));
        self.set_status_bit(Status::Zero, operand == 0);
        self.bus.write(address, operand);
        self.pc += 1;
    }

    pub(in crate::cpu) fn nop(&mut self, _imp: ()) {
        self.pc += 1;
    }

    pub(in crate::cpu) fn ora(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a |= operand;
        self.set_status_bit(Status::Zero, self.a == 0);
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    pub(in crate::cpu) fn pha(&mut self, _imp: ()) {
        self.push_stack(self.a);
        self.pc += 1;
    }

    pub(in crate::cpu) fn php(&mut self, _imp: ()) {
        self.push_stack(self.status | 0x30); // NES quirk, not regular 6502
        self.pc += 1;
    }

    pub(in crate::cpu) fn pla(&mut self, _imp: ()) {
        self.a = self.pop_stack();
        self.set_status_bit(Status::Zero, self.a == 0);
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    pub(in crate::cpu) fn plp(&mut self, _imp: ()) {
        self.status = (self.pop_stack() & 0xEF) | 0x20; // NES quirk, not regular 6502
        self.pc += 1;
    }

    pub(in crate::cpu) fn rol_acc(&mut self, _imp: ()) {
        let bit0 = self.is_set(Status::Carry) as u8;
        self.set_status_bit(Status::Carry, self.is_negative(self.a));
        self.a <<= 1;
        self.a |= bit0;
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.set_status_bit(Status::Zero, self.a == 0);
        self.pc += 1;
    }

    pub(in crate::cpu) fn rol_mem(&mut self, address: u16) {
        let operand = self.bus.read(address);
        let bit0 = self.is_set(Status::Carry) as u8;
        self.set_status_bit(Status::Carry, self.is_negative(operand));
        let operand = operand << 1;
        let operand = operand | bit0;
        self.bus.write(address, operand);
        self.set_status_bit(Status::Negative, self.is_negative(operand));
        self.set_status_bit(Status::Zero, operand == 0);
        self.pc += 1;
    }

    pub(in crate::cpu) fn ror_acc(&mut self, _imp: ()) {
        let bit7 = self.is_set(Status::Carry) as u8;
        self.set_status_bit(Status::Carry, self.a & 0x01 == 1);
        self.a >>= 1;
        self.a |= bit7 << 7;
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.set_status_bit(Status::Zero, self.a == 0);
        self.pc += 1;
    }

    pub(in crate::cpu) fn ror_mem(&mut self, address: u16) {
        let operand = self.bus.read(address);
        let bit7 = self.is_set(Status::Carry) as u8;
        self.set_status_bit(Status::Carry, operand & 0x01 == 1);
        let operand = operand >> 1;
        let operand = operand | bit7 << 7;
        self.bus.write(address, operand);
        self.set_status_bit(Status::Negative, self.is_negative(operand));
        self.set_status_bit(Status::Zero, operand == 0);
        self.pc += 1;
    }

    pub(in crate::cpu) fn rti(&mut self, _imp: ()) {
        self.status = self.pop_stack();
        let pcl = self.pop_stack();
        let pch = self.pop_stack();
        self.set_status_bit(Status::B2, true);
        self.pc = ((pch as u16) << 8) | pcl as u16;
    }

    pub(in crate::cpu) fn rts(&mut self, _imp: ()) {
        let pcl = self.pop_stack();
        let pch = self.pop_stack();
        self.pc = ((pch as u16) << 8) | pcl as u16;
        self.pc += 1;
    }

    pub(in crate::cpu) fn sbc(&mut self, address: u16) {
        let operand = self.bus.read(address) ^ 0xFF; // 2's complement (+1 nulified by 1-C)

        // rest is the same as adc
        let tmp = self.a as u16 + operand as u16 + self.is_set(Status::Carry) as u16;
        self.set_status_bit(Status::Carry, tmp > 0xFF);
        self.set_status_bit(Status::Zero, tmp & 0xFF == 0);
        self.set_status_bit(Status::Negative, (tmp & 0x80) >> 7 == 1);

        // overflows if positive + positive = negative or
        // negative + negative = positive
        // V = ~(A ^ OPERAND) & (A ^ TMP)
        self.set_status_bit(
            Status::Overflow,
            ((!(self.a as u16 ^ operand as u16) & (self.a as u16 ^ tmp)) & 0x0080) >> 7 == 1,
        );
        self.a = tmp as u8;
        self.pc += 1;
    }

    pub(in crate::cpu) fn sec(&mut self, _imp: ()) {
        self.set_status_bit(Status::Carry, true);
        self.pc += 1;
    }

    pub(in crate::cpu) fn sed(&mut self, _imp: ()) {
        self.set_status_bit(Status::Decimal, true);
        self.pc += 1;
    }

    pub(in crate::cpu) fn sei(&mut self, _imp: ()) {
        self.set_status_bit(Status::Interrupt, true);
        self.pc += 1;
    }

    pub(in crate::cpu) fn sta(&mut self, address: u16) {
        self.bus.write(address, self.a);
        self.pc += 1;
    }

    pub(in crate::cpu) fn stx(&mut self, address: u16) {
        self.bus.write(address, self.x);
        self.pc += 1;
    }

    pub(in crate::cpu) fn sty(&mut self, address: u16) {
        self.bus.write(address, self.y);
        self.pc += 1;
    }

    pub(in crate::cpu) fn tax(&mut self, _imp: ()) {
        self.x = self.a;
        self.set_status_bit(Status::Zero, self.x == 0);
        self.set_status_bit(Status::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn tay(&mut self, _imp: ()) {
        self.y = self.a;
        self.set_status_bit(Status::Zero, self.y == 0);
        self.set_status_bit(Status::Negative, (self.y & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn tsx(&mut self, _imp: ()) {
        self.x = self.sp as u8;
        self.set_status_bit(Status::Zero, self.x == 0);
        self.set_status_bit(Status::Negative, (self.x & 0x80) >> 7 == 1);
        self.pc += 1;
    }

    pub(in crate::cpu) fn txa(&mut self, _imp: ()) {
        self.a = self.x;
        self.set_status_bit(Status::Zero, self.a == 0);
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.pc += 1;
    }

    pub(in crate::cpu) fn txs(&mut self, _imp: ()) {
        self.sp = self.x;
        self.pc += 1;
    }

    pub(in crate::cpu) fn tya(&mut self, _imp: ()) {
        self.a = self.y;
        self.set_status_bit(Status::Zero, self.a == 0);
        self.set_status_bit(Status::Negative, self.is_negative(self.a));
        self.pc += 1;
    }
}

/// Unofficial instructions
impl<'a> CPU<'a> {
    pub(in crate::cpu) fn dcp(&mut self, address: u16) {
        let operand = self.bus.read(address).wrapping_sub(1);
        self.bus.write(address, operand);
        self.cmp(address);
    }

    pub(in crate::cpu) fn isc(&mut self, address: u16) {
        let operand = self.bus.read(address).wrapping_add(1);
        self.bus.write(address, operand);
        self.sbc(address);
    }

    pub(in crate::cpu) fn lax(&mut self, address: u16) {
        self.lda(address);
        self.pc -= 1;
        self.ldx(address);
    }

    pub(in crate::cpu) fn nop_unoff(&mut self, _: u16) {
        self.pc += 1;
    }

    pub(in crate::cpu) fn rla(&mut self, address: u16) {
        self.rol_mem(address);
        self.pc -= 1;
        self.and(address);
    }

    pub(in crate::cpu) fn rra(&mut self, address: u16) {
        self.ror_mem(address);
        self.pc -= 1;
        self.adc(address);
    }

    pub(in crate::cpu) fn sax(&mut self, address: u16) {
        self.bus.write(address, self.a & self.x);
        self.pc += 1;
    }

    pub(in crate::cpu) fn slo(&mut self, address: u16) {
        self.asl_mem(address);
        self.pc -= 1;
        self.ora(address);
    }

    pub(in crate::cpu) fn sre(&mut self, address: u16) {
        self.lsr_mem(address);
        self.pc -= 1;
        self.eor(address);
    }
}
