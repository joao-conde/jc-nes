use jc_nes::bus::Bus;
use jc_nes::cpu::CPU;
use jc_nes::ram::RAM;
use std::fs::File;
use std::io::Read;

fn main() {
    println!("Booting up NES...");

    // read test rom
    let mut rom = File::open("roms/nestest.nes").unwrap();
    let mut buffer = [0u8; 64 * 1024];
    rom.read(&mut buffer).expect("buffer overflow");

    // make test rom address start at 0xC000
    // and discard 16-bit header
    let mut mem = Vec::new();
    (0..0xC000).for_each(|_| mem.push(0));
    buffer[16..0x4F00]
        .into_iter()
        .for_each(|byte| mem.push(*byte));

    // connect ram to the bus
    // give bus to CPU to read/write
    let ram = RAM { mem };
    let mut bus = Bus::new();
    bus.connect(0x0000..=0xFFFF, ram);
    let mut cpu = CPU::new(bus);

    // emulate clock cycle
    while !cpu.terminated() {
        cpu.clock();
        // use std::io::stdin;
        // let mut s = String::new();
        // stdin().read_line(&mut s).unwrap();
    }

    // use jc_nes::bus::Device;
    // let res = cpu.read(0x0002);
    // println!("0x{:0x}", res);
    // let res = cpu.read(0x0003);
    // println!("0x{:0x}", res);
}
