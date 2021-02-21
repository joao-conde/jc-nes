use crate::bus::Device;

pub struct NameTable {
    mem: [u8; 1024],
}

impl NameTable {
    pub fn new() -> NameTable {
        NameTable { mem: [0u8; 1024] }
    }

    fn debug(&self) {
        // if self.mem.iter().all(|x| *x == 0x20) {
        //     return;
        // }
        // if self.mem.iter().all(|x| *x == 0x00) {
        //     return;
        // }
        // for i in 0..32 {
        //     for j in 0..32 {
        //         print!("{:02X} ", self.mem[i * 32 + j]);
        //     }
        //     println!();
        // }
        // self.pause();
    }

    fn pause(&self) {
        use std::io::stdin;
        let mut s = String::new();
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
    }
}

impl Device for NameTable {
    fn read(&mut self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
        self.debug();
    }
}
