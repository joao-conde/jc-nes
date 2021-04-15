use crate::cartridge::Cartridge;
use crate::{
    bus::{Device, SharedMut},
    cartridge::MirrorMode,
    ppu::PPU,
};
use std::{cell::RefCell, rc::Rc};

pub struct PrgMapper {
    ppu: SharedMut<PPU>,
    control: SharedMut<u8>,
    chr_bank_4_lo: SharedMut<usize>,
    chr_bank_4_hi: SharedMut<usize>,
    chr_bank_8: SharedMut<usize>,

    prg_bank_16_lo: usize,
    prg_bank_16_hi: usize,
    prg_bank_32: usize,
    prg_mem: Vec<u8>,
    prg_banks: usize,
    load: u8,
    load_cnt: usize,
}

pub struct ChrMapper {
    control: SharedMut<u8>,
    chr_bank_4_lo: SharedMut<usize>,
    chr_bank_4_hi: SharedMut<usize>,
    chr_bank_8: SharedMut<usize>,

    chr_mem: Vec<u8>,
}

pub fn new_mapper(cartridge: Cartridge, ppu: SharedMut<PPU>) -> (PrgMapper, ChrMapper) {
    let control = Rc::new(RefCell::new(0x1C));
    let chr_bank_4_lo = Rc::new(RefCell::new(0));
    let chr_bank_4_hi = Rc::new(RefCell::new(0));
    let chr_bank_8 = Rc::new(RefCell::new(0));

    let prg_mapper = PrgMapper {
        control: control.clone(),
        ppu: ppu,
        chr_bank_4_lo: chr_bank_4_lo.clone(),
        chr_bank_4_hi: chr_bank_4_hi.clone(),
        chr_bank_8: chr_bank_8.clone(),

        prg_bank_16_lo: 0,
        prg_bank_16_hi: cartridge.prg_banks - 1,
        prg_bank_32: 0,
        prg_mem: cartridge.prg_rom,
        prg_banks: cartridge.prg_banks,

        load: 0x00,
        load_cnt: 0,
    };
    let chr_mapper = ChrMapper {
        control: control,
        chr_bank_4_lo: chr_bank_4_lo,
        chr_bank_4_hi: chr_bank_4_hi,
        chr_bank_8: chr_bank_8,

        chr_mem: if cartridge.chr_banks == 0 {
            [0u8; 8 * 1024].to_vec()
        } else {
            cartridge.chr_rom
        },
    };
    (prg_mapper, chr_mapper)
}

impl PrgMapper {
    fn map_address(&self, address: u16) -> u16 {
        if *self.control.borrow() & 0b10000 != 0 {
            // 16K PRG Bank Mode
            if address <= 0x3FFF {
                self.prg_bank_16_lo as u16 * 0x4000 + (address & 0x3FFF)
            } else {
                self.prg_bank_16_hi as u16 * 0x4000 + (address & 0x3FFF)
            }
        } else {
            // 32K PRG Bank Mode
            self.prg_bank_32 as u16 * 0x8000 + (address & 0x7FFF)
        }
    }
}

impl ChrMapper {
    fn map_address(&self, address: u16) -> u16 {
        if *self.control.borrow() & 0b10000 != 0 {
            // 4K CHR Bank Mode
            if address <= 0x0FFF {
                *self.chr_bank_4_lo.borrow() as u16 * 0x1000 + (address & 0x0FFF)
            } else {
                *self.chr_bank_4_hi.borrow() as u16 * 0x1000 + (address & 0x0FFF)
            }
        } else {
            // 8K CHR Bank Mode
            *self.chr_bank_8.borrow() as u16 * 0x2000 + (address & 0x1FFF)
        }
    }
}

impl Device for PrgMapper {
    fn read(&mut self, address: u16) -> u8 {
        let address = self.map_address(address);
        self.prg_mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        if data & 0x80 != 0 {
            self.load = 0x00;
            self.load_cnt = 0;

            let control_byte = *self.control.borrow();
            self.control.replace(control_byte | 0x0C);
        } else {
            // load data serially (LSB first) into load register
            self.load >>= 1;
            self.load |= (data & 0x01) << 4;
            self.load_cnt += 1;

            if self.load_cnt == 5 {
                let target_register = ((address + 0x8000) >> 13) & 0x03;

                // 0x8000 - 0x9FFF
                if target_register == 0 {
                    self.control.replace(self.load & 0x1F);
                    match *self.control.borrow() & 0x03 {
                        0 => self.ppu.borrow_mut().mirror_mode = MirrorMode::OneScreenLo,
                        1 => self.ppu.borrow_mut().mirror_mode = MirrorMode::OneScreenHi,
                        2 => self.ppu.borrow_mut().mirror_mode = MirrorMode::Vertical,
                        3 => self.ppu.borrow_mut().mirror_mode = MirrorMode::Horizontal,
                        _ => (),
                    };
                }
                // 0xA000 - 0xBFFF
                else if target_register == 1 {
                    if *self.control.borrow() & 0b10000 != 0 {
                        self.chr_bank_4_lo.replace((self.load & 0x1F) as usize);
                    } else {
                        self.chr_bank_8.replace((self.load & 0x1E) as usize);
                    }
                } else if target_register == 2
                // 0xC000 - 0xDFFF
                {
                    if *self.control.borrow() & 0b10000 != 0 {
                        self.chr_bank_4_hi.replace((self.load & 0x1F) as usize);
                    }
                } else if target_register == 3
                // 0xE000 - 0xFFFF
                {
                    let prg_mode = (*self.control.borrow() >> 2) & 0x03;
                    if prg_mode == 0 || prg_mode == 1 {
                        self.prg_bank_32 = ((self.load & 0x0E) >> 1) as usize;
                    } else if prg_mode == 2 {
                        self.prg_bank_16_lo = 0;
                        self.prg_bank_16_hi = (self.load & 0x0F) as usize;
                    } else if prg_mode == 3 {
                        self.prg_bank_16_lo = (self.load & 0x0F) as usize;
                        self.prg_bank_16_hi = self.prg_banks - 1;
                    }
                }

                // reset load register
                self.load = 0x00;
                self.load_cnt = 0;
            }
        }
    }
}

impl Device for ChrMapper {
    fn read(&mut self, address: u16) -> u8 {
        let address = self.map_address(address);
        self.chr_mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        let address = self.map_address(address);
        self.chr_mem[address as usize] = data;
    }
}
