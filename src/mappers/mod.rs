pub mod mapper000;

// aghhh... how will I differentiate from PPU or CPU mapping if read comes from both...
// bus created with type? defeats purpose of generic bus tho
trait Mapper {
    fn map_cpu_read(address: u16) -> u16;
    fn map_cpu_write(address: u16) -> u16;
    fn map_ppu_read(address: u16) -> u16;
    fn map_ppu_write(address: u16) -> u16;
}
