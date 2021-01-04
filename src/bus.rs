use std::{collections::HashMap, ops::RangeInclusive};

#[derive(Default)]
pub struct Bus<'a> {
    pub addresses: HashMap<RangeInclusive<u16>, &'a mut dyn Device>,
}

impl<'a> Bus<'a> {
    pub fn connect(&mut self, addressable_range: RangeInclusive<u16>, device: &'a mut impl Device) {
        self.addresses.insert(addressable_range, device);
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
}

pub trait Device {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}
