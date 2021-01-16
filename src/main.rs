use jc_nes::bus::Bus;
use jc_nes::cpu::CPU;
use jc_nes::ram::RAM;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

fn main() {
    emulate()
}

fn emulate() {
    let ram = Rc::new(RefCell::new(RAM::new(2 * 1024)));

    let mut cpu_bus = Bus::default();
    cpu_bus.connect(0x0000..=0x1FFF, &ram);

    let mut ppu_bus = Bus::default();

    let mut cpu = CPU::new(&mut cpu_bus);
}

fn nestest() {
    // read test rom
    let mut rom = File::open("roms/nestest.nes").unwrap();
    let mut buffer = [0u8; 64 * 1024];
    rom.read(&mut buffer).expect("buffer overflow");

    // make test rom address start at 0xC000
    // and discard 16-bit header
    let mut mem = Vec::new();
    (0..0xC000).for_each(|_| mem.push(0));
    buffer[16..0x4F00].iter().for_each(|byte| mem.push(*byte));

    // connect ram to the bus
    // give bus to CPU to read/write
    let ram = Rc::new(RefCell::new(RAM { mem }));
    let mut bus = Bus::default();
    bus.connect(0x0000..=0xFFFF, &ram);

    let mut cpu = CPU::new(&mut bus);

    // emulate clock cycle
    for _ in 0..26548 {
        cpu.clock();
    }
}
