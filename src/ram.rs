use super::bus::Device;
// TODO remove test RAM device
pub struct RAM {
    pub mem: [u8; 64 * 1024],
}

impl Device for RAM {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize % 10]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize % 10] = data;
    }

    fn print(&self) {
        println!("---RAM---");
        for (address, byte) in self.mem.iter().enumerate() {
            println!("0x{:0x} 0x{:0x}", address, byte);
        }
    }
}
