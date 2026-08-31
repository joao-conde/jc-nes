use crate::{
    bus::Device,
    cpu::{Cpu, Status},
};

/// Instructions
impl Cpu {
    pub(in crate::cpu) fn adc(&mut self, address: u16) {
        let operand = self.bus.read(address);
        let tmp = self.a as u16 + operand as u16 + self.status.carry as u16;

        self.status.carry = tmp > 0xFF;
        self.status.zero = tmp & 0xFF == 0;
        self.status.negative = self.is_negative((tmp & 0xFF) as u8);

        // OVERFLOWs if positive + positive = NEGATIVE or
        // NEGATIVE + NEGATIVE = positive
        // V = ~(A ^ OPERAND) & (A ^ TMP)
        self.status.overflow =
            ((!(self.a as u16 ^ operand as u16) & (self.a as u16 ^ tmp)) & 0x0080) >> 7 == 1;
        self.a = tmp as u8;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn and(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a &= operand;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn asl_acc(&mut self, _acc: ()) {
        self.status.carry = self.is_negative(self.a);
        self.a <<= 1;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn asl_mem(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.status.carry = self.is_negative(operand);
        let operand = operand << 1;
        self.status.negative = self.is_negative(operand);
        self.status.zero = operand == 0;
        self.bus.write(address, operand);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn bcc(&mut self, address: u16) {
        let opcode = self.bus.read(address);
        self.relative_jump(!self.status.carry, opcode as i8);
    }

    pub(in crate::cpu) fn bcs(&mut self, address: u16) {
        let opcode = self.bus.read(address);
        self.relative_jump(self.status.carry, opcode as i8);
    }

    pub(in crate::cpu) fn beq(&mut self, address: u16) {
        let opcode = self.bus.read(address);
        self.relative_jump(self.status.zero, opcode as i8);
    }

    pub(in crate::cpu) fn bit(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.status.zero = self.a & operand == 0;
        self.status.negative = self.is_negative(operand);
        self.status.overflow = (operand & 0x40) >> 6 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn bmi(&mut self, address: u16) {
        let opcode = self.bus.read(address);
        self.relative_jump(self.status.negative, opcode as i8);
    }

    pub(in crate::cpu) fn bne(&mut self, address: u16) {
        let opcode = self.bus.read(address);
        self.relative_jump(!self.status.zero, opcode as i8);
    }

    pub(in crate::cpu) fn bpl(&mut self, address: u16) {
        let opcode = self.bus.read(address);
        self.relative_jump(!self.status.negative, opcode as i8);
    }

    pub(in crate::cpu) fn brk(&mut self, _imp: ()) {
        // BRK is one byte but skips two: the padding byte after the opcode is
        // fetched and discarded, so the return address is the instruction after
        // that. This is why an RTI from a BRK handler lands correctly.
        let return_address = self.pc.wrapping_add(2);
        self.push_stack((return_address >> 8) as u8);
        self.push_stack(return_address as u8);

        // The B flag does not exist in the register; it is synthesised as set
        // only in the byte an interrupt or PHP pushes, alongside bit 5.
        self.push_stack(u8::from(self.status) | 0x30);
        self.status.interrupt = true;

        let pcl = self.bus.read(0xFFFE);
        let pch = self.bus.read(0xFFFF);
        self.pc = ((pch as u16) << 8) | pcl as u16;
    }

    pub(in crate::cpu) fn bvc(&mut self, address: u16) {
        let opcode = self.bus.read(address);
        self.relative_jump(!self.status.overflow, opcode as i8);
    }

    pub(in crate::cpu) fn bvs(&mut self, address: u16) {
        let opcode = self.bus.read(address);
        self.relative_jump(self.status.overflow, opcode as i8);
    }

    pub(in crate::cpu) fn clc(&mut self, _imp: ()) {
        self.status.carry = false;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn cld(&mut self, _imp: ()) {
        self.status.decimal = false;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn cli(&mut self, _imp: ()) {
        self.status.interrupt = false;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn clv(&mut self, _imp: ()) {
        self.status.overflow = false;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn cmp(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.status.carry = self.a >= operand;
        self.status.zero = self.a == operand;
        self.status.negative = (self.a.wrapping_sub(operand) & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn cpx(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.status.carry = self.x >= operand;
        self.status.zero = self.x == operand;
        self.status.negative = (self.x.wrapping_sub(operand) & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn cpy(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.status.carry = self.y >= operand;
        self.status.zero = self.y == operand;
        self.status.negative = (self.y.wrapping_sub(operand) & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn dec(&mut self, address: u16) {
        let operand = self.bus.read(address).wrapping_sub(1);
        self.bus.write(address, operand);
        self.status.zero = operand == 0;
        self.status.negative = self.is_negative(operand);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn dex(&mut self, _imp: ()) {
        self.x = self.x.wrapping_sub(1);
        self.status.zero = self.x == 0;
        self.status.negative = (self.x & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn dey(&mut self, _imp: ()) {
        self.y = self.y.wrapping_sub(1);
        self.status.zero = self.y == 0;
        self.status.negative = (self.y & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn eor(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a ^= operand;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn inc(&mut self, address: u16) {
        let operand = self.bus.read(address);
        let operand = operand.wrapping_add(1);
        self.bus.write(address, operand);
        self.status.zero = operand == 0;
        self.status.negative = self.is_negative(operand);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn inx(&mut self, _imp: ()) {
        self.x = self.x.wrapping_add(1);
        self.status.zero = self.x == 0;
        self.status.negative = (self.x & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn iny(&mut self, _imp: ()) {
        self.y = self.y.wrapping_add(1);
        self.status.zero = self.y == 0;
        self.status.negative = (self.y & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
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
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn ldx(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.x = operand;
        self.status.zero = self.x == 0;
        self.status.negative = (self.x & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn ldy(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.y = operand;
        self.status.zero = self.y == 0;
        self.status.negative = (self.y & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn lsr_acc(&mut self, _acc: ()) {
        self.status.carry = self.a & 0x01 == 1;
        self.a >>= 1;
        self.status.negative = self.is_negative(self.a);
        self.status.zero = self.a == 0;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn lsr_mem(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.status.carry = operand & 0x01 == 1;
        let operand = operand >> 1;
        self.status.negative = self.is_negative(operand);
        self.status.zero = operand == 0;
        self.bus.write(address, operand);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn nop(&mut self, _imp: ()) {
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn ora(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a |= operand;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn pha(&mut self, _imp: ()) {
        self.push_stack(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn php(&mut self, _imp: ()) {
        self.push_stack(u8::from(self.status) | 0x30); // NES quirk, not regular 6502
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn pla(&mut self, _imp: ()) {
        self.a = self.pop_stack();
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn plp(&mut self, _imp: ()) {
        self.status = Status::from((self.pop_stack() & 0xEF) | 0x20); // NES quirk, not regular 6502
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn rol_acc(&mut self, _imp: ()) {
        let bit0 = self.status.carry as u8;
        self.status.carry = self.is_negative(self.a);
        self.a <<= 1;
        self.a |= bit0;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn rol_mem(&mut self, address: u16) {
        let operand = self.bus.read(address);
        let bit0 = self.status.carry as u8;
        self.status.carry = self.is_negative(operand);
        let operand = operand << 1;
        let operand = operand | bit0;
        self.bus.write(address, operand);
        self.status.negative = self.is_negative(operand);
        self.status.zero = operand == 0;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn ror_acc(&mut self, _imp: ()) {
        let bit7 = self.status.carry as u8;
        self.status.carry = self.a & 0x01 == 1;
        self.a >>= 1;
        self.a |= bit7 << 7;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn ror_mem(&mut self, address: u16) {
        let operand = self.bus.read(address);
        let bit7 = self.status.carry as u8;
        self.status.carry = operand & 0x01 == 1;
        let operand = operand >> 1;
        let operand = operand | bit7 << 7;
        self.bus.write(address, operand);
        self.status.negative = self.is_negative(operand);
        self.status.zero = operand == 0;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn rti(&mut self, _imp: ()) {
        // Like PLP, RTI ignores bit 4 of the pulled byte and forces bit 5:
        // neither is real storage in the status register.
        self.status = Status::from((self.pop_stack() & 0xEF) | 0x20);
        let pcl = self.pop_stack();
        let pch = self.pop_stack();
        self.pc = ((pch as u16) << 8) | pcl as u16;
    }

    pub(in crate::cpu) fn rts(&mut self, _imp: ()) {
        let pcl = self.pop_stack();
        let pch = self.pop_stack();
        self.pc = ((pch as u16) << 8) | pcl as u16;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn sbc(&mut self, address: u16) {
        let operand = self.bus.read(address) ^ 0xFF; // 2's complement (+1 nulified by 1-C)

        // rest is the same as adc
        let tmp = self.a as u16 + operand as u16 + self.status.carry as u16;
        self.status.carry = tmp > 0xFF;
        self.status.zero = tmp & 0xFF == 0;
        self.status.negative = (tmp & 0x80) >> 7 == 1;

        // OVERFLOWs if positive + positive = NEGATIVE or
        // NEGATIVE + NEGATIVE = positive
        // V = ~(A ^ OPERAND) & (A ^ TMP)
        self.status.overflow =
            ((!(self.a as u16 ^ operand as u16) & (self.a as u16 ^ tmp)) & 0x0080) >> 7 == 1;
        self.a = tmp as u8;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn sec(&mut self, _imp: ()) {
        self.status.carry = true;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn sed(&mut self, _imp: ()) {
        self.status.decimal = true;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn sei(&mut self, _imp: ()) {
        self.status.interrupt = true;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn sta(&mut self, address: u16) {
        self.bus.write(address, self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn stx(&mut self, address: u16) {
        self.bus.write(address, self.x);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn sty(&mut self, address: u16) {
        self.bus.write(address, self.y);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn tax(&mut self, _imp: ()) {
        self.x = self.a;
        self.status.zero = self.x == 0;
        self.status.negative = (self.x & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn tay(&mut self, _imp: ()) {
        self.y = self.a;
        self.status.zero = self.y == 0;
        self.status.negative = (self.y & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn tsx(&mut self, _imp: ()) {
        self.x = self.sp;
        self.status.zero = self.x == 0;
        self.status.negative = (self.x & 0x80) >> 7 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn txa(&mut self, _imp: ()) {
        self.a = self.x;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn txs(&mut self, _imp: ()) {
        self.sp = self.x;
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn tya(&mut self, _imp: ()) {
        self.a = self.y;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }
}

/// Unofficial instructions
impl Cpu {
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
        self.pc = self.pc.wrapping_sub(1);
        self.ldx(address);
    }

    /// LXA ($AB), the immediate form of LAX.
    ///
    /// Unstable on real silicon: A and X both take `(A | magic) & imm` rather
    /// than the immediate alone. `0xEE` is the only constant in `0..=0xFF`
    /// consistent with all 10,000 reference cases for this opcode.
    pub(in crate::cpu) fn lxa(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a = (self.a | 0xEE) & operand;
        self.x = self.a;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn nop_unoff(&mut self, _: u16) {
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn rla(&mut self, address: u16) {
        self.rol_mem(address);
        self.pc = self.pc.wrapping_sub(1);
        self.and(address);
    }

    pub(in crate::cpu) fn rra(&mut self, address: u16) {
        self.ror_mem(address);
        self.pc = self.pc.wrapping_sub(1);
        self.adc(address);
    }

    pub(in crate::cpu) fn sax(&mut self, address: u16) {
        self.bus.write(address, self.a & self.x);
        self.pc = self.pc.wrapping_add(1);
    }

    pub(in crate::cpu) fn slo(&mut self, address: u16) {
        self.asl_mem(address);
        self.pc = self.pc.wrapping_sub(1);
        self.ora(address);
    }

    pub(in crate::cpu) fn sre(&mut self, address: u16) {
        self.lsr_mem(address);
        self.pc = self.pc.wrapping_sub(1);
        self.eor(address);
    }

    /// ANC ($0B, $2B): AND immediate, then copy bit 7 of the result into carry.
    pub(in crate::cpu) fn anc(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a &= operand;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.status.carry = self.status.negative;
        self.pc = self.pc.wrapping_add(1);
    }

    /// ALR ($4B): AND immediate, then LSR the accumulator.
    pub(in crate::cpu) fn alr(&mut self, address: u16) {
        let operand = self.bus.read(address) & self.a;
        self.status.carry = operand & 0x01 == 1;
        self.a = operand >> 1;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    /// ARR ($6B): AND immediate, then ROR, with carry and overflow taken from
    /// the rotated result rather than from the shifted-out bit.
    pub(in crate::cpu) fn arr(&mut self, address: u16) {
        let operand = self.bus.read(address) & self.a;
        self.a = (operand >> 1) | ((self.status.carry as u8) << 7);
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.status.carry = self.a & 0x40 != 0;
        self.status.overflow = ((self.a >> 6) ^ (self.a >> 5)) & 0x01 == 1;
        self.pc = self.pc.wrapping_add(1);
    }

    /// ANE/XAA ($8B), unstable: shares LXA's `0xEE` magic constant.
    pub(in crate::cpu) fn xaa(&mut self, address: u16) {
        let operand = self.bus.read(address);
        self.a = (self.a | 0xEE) & self.x & operand;
        self.status.zero = self.a == 0;
        self.status.negative = self.is_negative(self.a);
        self.pc = self.pc.wrapping_add(1);
    }

    /// SBX/AXS ($CB): X = (A & X) - immediate, with carry set like a compare.
    pub(in crate::cpu) fn sbx(&mut self, address: u16) {
        let operand = self.bus.read(address);
        let masked = self.a & self.x;
        self.x = masked.wrapping_sub(operand);
        self.status.carry = masked >= operand;
        self.status.zero = self.x == 0;
        self.status.negative = self.is_negative(self.x);
        self.pc = self.pc.wrapping_add(1);
    }

    /// LAS ($BB): A, X and S all take memory ANDed with the stack pointer.
    pub(in crate::cpu) fn las(&mut self, address: u16) {
        let result = self.bus.read(address) & self.sp;
        self.a = result;
        self.x = result;
        self.sp = result;
        self.status.zero = result == 0;
        self.status.negative = self.is_negative(result);
        self.pc = self.pc.wrapping_add(1);
    }

    /// SHY ($9C): store Y ANDed with the target's high byte plus one.
    pub(in crate::cpu) fn shy(&mut self, base: u16) {
        let value = self.y & Cpu::unstable_mask(base);
        let target = Cpu::unstable_target(base, self.x, value);
        self.bus.write(target, value);
        self.pc = self.pc.wrapping_add(1);
    }

    /// SHX ($9E): store X ANDed with the target's high byte plus one.
    pub(in crate::cpu) fn shx(&mut self, base: u16) {
        let value = self.x & Cpu::unstable_mask(base);
        let target = Cpu::unstable_target(base, self.y, value);
        self.bus.write(target, value);
        self.pc = self.pc.wrapping_add(1);
    }

    /// SHA ($93, $9F): store A & X ANDed with the target's high byte plus one.
    pub(in crate::cpu) fn sha(&mut self, base: u16) {
        let value = self.a & self.x & Cpu::unstable_mask(base);
        let target = Cpu::unstable_target(base, self.y, value);
        self.bus.write(target, value);
        self.pc = self.pc.wrapping_add(1);
    }

    /// TAS ($9B): S takes A & X, then that is stored the way SHA stores.
    pub(in crate::cpu) fn tas(&mut self, base: u16) {
        self.sp = self.a & self.x;
        let value = self.sp & Cpu::unstable_mask(base);
        let target = Cpu::unstable_target(base, self.y, value);
        self.bus.write(target, value);
        self.pc = self.pc.wrapping_add(1);
    }
}
