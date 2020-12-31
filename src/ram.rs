use super::bus::Device;
// TODO remove test RAM device
pub struct RAM {
    pub mem: Vec<u8>,
}

impl Device for RAM {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.mem[address as usize] = data;
    }
}
