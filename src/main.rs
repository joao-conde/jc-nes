use jc_nes::bus::Bus;

fn main() {
    println!("Booting up NES...");
    let bus = Bus::new();
    bus.connect();
    bus.read(0xFF);
    bus.write(0xFF);
}
