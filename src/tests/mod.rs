use std::fs::File;
use std::io::Read;
use crate::ram::RAM;
use crate::bus::Bus;
use crate::cpu::CPU;

const NESTEST_ROM: &str = "nestest.nes";
const NESTEST_LOG: &str = "nestest.log";
const NESTEST_CLOCK_CNT: usize = 26548;

#[test]
fn test_with_nestest() {
    let rom = load_nestest_rom(NESTEST_ROM);

    // connect ram to the bus
    let mut ram = RAM { mem: rom };
    let mut bus = Bus::default();
    bus.connect(0x0000..=0xFFFF, &mut ram);

    let mut cpu = CPU::new(&mut bus);

    // emulate clock cycles
    for _ in 0..NESTEST_CLOCK_CNT {
        cpu.clock();
    }
}

fn load_nestest_rom(path: &str) -> Vec<u8> {
    let mut rom = File::open(NESTEST_ROM).expect("failure opening nestest ROM");
    let mut buffer = [0u8; 64 * 1024];
    rom.read(&mut buffer).expect("buffer overflow");

    // make test rom address start at 0xC000 and discard 16-bit header
    let mut rom = Vec::new();
    (0..0xC000).for_each(|_| rom.push(0));
    buffer[16..0x4F00].iter().for_each(|byte| rom.push(*byte));

    rom
}