use std::{collections::HashMap, ops::RangeInclusive};

pub struct Bus<'a> {
    pub addresses: HashMap<RangeInclusive<u16>, Box<dyn Device + 'a>>,
}

impl<'a> Bus<'a> {
    pub fn new() -> Bus<'a> {
        Bus {
            addresses: HashMap::new(),
        }
    }

    pub fn connect(&mut self, addressable_range: RangeInclusive<u16>, device: impl Device + 'a) {
        self.addresses.insert(addressable_range, Box::new(device));
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        self.addresses
            .iter()
            .filter(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(_, device)| device.read(address))
            .next()
    }

    pub fn write(&mut self, address: u16, data: u8) {
        for (addressable_range, device) in &mut self.addresses {
            if addressable_range.contains(&address) {
                device.write(address, data);
            }
        }
    }

    pub fn print_devices(&self) {
        &self.addresses.values().for_each(|device| device.print());
    }
}

pub trait Device {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
    fn print(&self);
}
