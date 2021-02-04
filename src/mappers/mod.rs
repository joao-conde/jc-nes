pub mod mapper0;

trait Mapper {
    fn map_cpu_read(address: u16) -> u16;
    fn map_cpu_write(address: u16) -> u16;
    fn map_ppu_read(address: u16) -> u16;
    fn map_ppu_write(address: u16) -> u16;
}
