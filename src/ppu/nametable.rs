use crate::bus::Device;

pub struct NameTable {
    mem: [u8; 1024],
}

impl NameTable {
    pub fn new() -> NameTable {
        NameTable { mem: [0u8; 1024] }
    }
}

impl Device for NameTable {
    fn read(&mut self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        // println!("Writing to nametable addr:0x{:X} with data:0x{:X}", address, data);
        self.mem[address as usize] = data;
    }
}
