use super::bus::Device;
pub struct CPU {

}

impl Device for CPU {
    fn read(&self, address: usize) -> usize {
        println!("CPU reading from {:0x}!", address);
        0xAB
    }

    fn write(&self, address: usize) {
        println!("CPU writing to {:0x}", address)
    }
}