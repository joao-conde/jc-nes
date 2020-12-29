use jc_nes::bus::Bus;
use jc_nes::cpu::CPU;
use jc_nes::ram::RAM;

fn main() {
    println!("Booting up NES...");

    let mut bus = Bus::new();

    let cpu = CPU::new(&bus);

    
    let ram = RAM { mem: [0; 64 * 1024] };

    bus.connect(0x0000..=0xFFFF, ram);

}
