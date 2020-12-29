use jc_nes::bus::Bus;
use jc_nes::cpu::CPU;

fn main() {
    println!("Booting up NES...");

    let cpu = CPU {};
    let mut bus = Bus::new();
    
    bus.connect(0x00..=0xAA, cpu);

    let valid_read = bus.read(0x00);
    println!("Valid read {:0x?}", valid_read);
    let valid_read = bus.read(0x26);
    println!("Valid read {:0x?}", valid_read);
    let valid_read = bus.read(0xAA);
    println!("Valid read {:0x?}", valid_read);
    let invalid_read = bus.read(0xFF);
    println!("Invalid read {:0x?}", invalid_read);
    bus.write(0x1A);
}
