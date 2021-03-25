use crate::cpu::CPU;

/// Addressing Modes
impl CPU {
    pub(in crate::cpu) fn abs(&mut self) -> u16 {
        self.pc += 1;
        let lo = self.bus.read(self.pc);
        self.pc += 1;
        let hi = self.bus.read(self.pc);
        ((hi as u16) << 8) | lo as u16
    }

    pub(in crate::cpu) fn absx(&mut self) -> u16 {
        let address = self.abs();
        let hi = address & 0xFF00;
        let address = address.wrapping_add(self.x as u16);
        self.cycle += (self.extra_cycles && self.page_crossed(hi, address)) as u8;
        address
    }

    pub(in crate::cpu) fn absy(&mut self) -> u16 {
        let address = self.abs();
        let hi = address & 0xFF00;
        let address = address.wrapping_add(self.y as u16);
        self.cycle += (self.extra_cycles && self.page_crossed(hi, address)) as u8;
        address
    }

    pub(in crate::cpu) fn acc(&mut self) {}

    pub(in crate::cpu) fn imm(&mut self) -> u16 {
        self.pc += 1;
        self.pc
    }

    pub(in crate::cpu) fn imp(&mut self) {}

    pub(in crate::cpu) fn ind(&mut self) -> u16 {
        self.pc += 1;
        let lo = self.bus.read(self.pc);
        self.pc += 1;
        let hi = self.bus.read(self.pc);
        let address = ((hi as u16) << 8) | lo as u16;

        // "ind" is bugged in the original hardware
        // if the low byte is 0xFF then the high byte should be read from the next page
        // the bug is that it does not, and instead just wraps around in the same page
        if lo == 0xFF {
            ((self.bus.read(address & 0xFF00) as u16) << 8) | self.bus.read(address) as u16
        } else {
            ((self.bus.read(address + 1) as u16) << 8) | self.bus.read(address) as u16
        }
    }

    pub(in crate::cpu) fn indx(&mut self) -> u16 {
        self.pc += 1;
        let address = self.bus.read(self.pc) as u16;
        let lo = self.bus.read((address + self.x as u16) & 0x00FF);
        let hi = self.bus.read((address + 1 + self.x as u16) & 0x00FF);
        ((hi as u16) << 8) + lo as u16
    }

    pub(in crate::cpu) fn indy(&mut self) -> u16 {
        self.pc += 1;
        let address = self.bus.read(self.pc) as u16;
        let lo = self.bus.read(address & 0x00FF) as u16;
        let hi = self.bus.read((address + 1) & 0x00FF);
        let hi = (hi as u16) << 8;
        let address = lo.wrapping_add(hi).wrapping_add(self.y as u16);
        self.cycle += (self.extra_cycles && self.page_crossed(address, hi)) as u8;
        address
    }

    pub(in crate::cpu) fn relative(&mut self) -> u16 {
        self.pc += 1;
        self.pc
    }

    pub(in crate::cpu) fn zp(&mut self) -> u16 {
        self.pc += 1;
        self.bus.read(self.pc) as u16
    }

    pub(in crate::cpu) fn zpx(&mut self) -> u16 {
        (self.zp() + self.x as u16) & 0xFF
    }

    pub(in crate::cpu) fn zpy(&mut self) -> u16 {
        (self.zp() + self.y as u16) & 0xFF
    }
}
