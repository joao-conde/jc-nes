pub struct PPU {
    name_tables: [u8; 8 * 1024],
    palette: [u8; 255],
}

impl PPU {
    pub fn clock(&self) {
        println!("clock PPU");
    }
}
