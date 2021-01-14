use std::{collections::HashMap, ops::RangeInclusive};

pub trait Device {
    fn read(&self, address: u16) -> u8 {
        panic!(format!("read from 0x{:04X} not allowed because device not readable", address));
    }

    fn write(&mut self, address: u16, data: u8) {
        panic!(format!(
            "write of 0x{:02X} to 0x{:04X} not allowed because device not writable",
            data, address
        ));
    }
}

#[derive(Default)]
pub struct Bus<'a> {
    pub addresses: HashMap<RangeInclusive<u16>, &'a mut dyn Device>,
}

impl<'a> Bus<'a> {
    pub fn connect(&mut self, addressable_range: RangeInclusive<u16>, device: &'a mut impl Device) {
        match self.addresses.iter().any(|(range, _)| overlaps(range, &addressable_range)) {
            false => self.addresses.insert(addressable_range, device),
            true => panic!("can not connect device because addressable range already exists"),
        };
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        let device = self
            .addresses
            .iter()
            .filter(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(_, device)| device)
            .next();

        device.map(|device| device.read(address))
    }

    pub fn write(&mut self, address: u16, data: u8) {
        let device = self
            .addresses
            .iter_mut()
            .filter(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(_, device)| device)
            .next();

        device.map(|device| device.write(address, data));
    }
}

fn overlaps(range1: &RangeInclusive<u16>, range2: &RangeInclusive<u16>) -> bool {
    !(range1.end() < range2.start() || range2.end() < range1.start())
}
