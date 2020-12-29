use std::{collections::HashMap, ops::{RangeInclusive}};

pub struct Bus<'a> {
    pub addresses: HashMap<RangeInclusive<usize>, Box<dyn Device + 'a>>,
}

impl<'a> Bus<'a> {
    pub fn new() -> Bus<'a> {
        Bus {
            addresses: HashMap::new(),
        }
    }

    pub fn connect(&mut self, addressable_range: RangeInclusive<usize>, device: impl Device + 'a) {
        self.addresses.insert(addressable_range, Box::new(device));
    }

    pub fn read(&self, address: usize) -> Option<usize> {
        self.addresses
            .iter()
            .filter(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(_, device)| device.read(address))
            .next()
    }

    pub fn write(&self, address: usize) {
        for (addressable_range, device) in &self.addresses {
            if addressable_range.contains(&address) {
                device.write(address);
            }
        }
    }
}

pub trait Device {
    fn read(&self, address: usize) -> usize;
    fn write(&self, address: usize);
}
