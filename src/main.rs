use jc_nes::bus::Bus;
use jc_nes::cpu::CPU;
use jc_nes::ram::RAM;
use std::fs::File;
use std::io::Read;

fn main() {
    println!("Booting up NES...");

    let mut rom = File::open("roms/nestest.nes").unwrap();
    let mut buffer = vec![0; 64 * 1024];
    rom.read(&mut buffer).expect("buffer overflow");
    let ram = RAM {
        mem: [0; 64 * 1024],
    };

    let mut bus = Bus::new();
    bus.connect(0x0000..=0xFFFF, ram);

    let mut cpu = CPU::new(&bus);

    loop {
        cpu.clock();
    }
}
