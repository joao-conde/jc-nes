use jc_nes::bus::Bus;
use jc_nes::cpu::CPU;
use jc_nes::ram::RAM;
use std::fs::File;
use std::io::Read;

fn main() {
    println!("Booting up NES...");

    let mut bus = Bus::new();

    let cpu = CPU::new(&bus);

    let mut rom = File::open("roms/nestest.nes").unwrap();
    let mut buffer = vec![0; 64 * 1024];

    rom.read(&mut buffer).expect("buffer overflow");
    let ram = RAM {
        mem: [0; 64 * 1024],
    };

    bus.connect(0x0000..=0xFFFF, ram);

    println!(
        "{:0b} {:0b} {:0b}",
        0b11110000,
        0b11110000 & 0b1111,
        0b11110000 >> 4
    );
}
