use jc_nes::bus::Bus;

fn main() {
    println!("Booting up NES...");
    let bus = Bus {};
    bus.connect();
    bus.read();
    bus.write();
    println!("{:?}", bus)
}
