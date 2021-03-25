use std::{collections::HashMap, ops::RangeInclusive};

#[derive(Default)]
pub struct Bus {
    devices: HashMap<RangeInclusive<u16>, Box<dyn Device>>,
    mirrors: HashMap<RangeInclusive<u16>, u16>,
}

pub trait Device {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

impl Bus {
    pub fn connect(
        &mut self,
        addressable_range: RangeInclusive<u16>,
        device: impl Device + 'static,
    ) {
        self.devices
            .insert(addressable_range, Box::new(device));
    }

    pub fn add_mirror(&mut self, addressable_range: RangeInclusive<u16>, max: u16) {
        self.mirrors.insert(addressable_range, max);
    }

    pub fn read(&mut self, address: u16) -> u8 {
        let address = self.mirror(address);
        self.devices
            .iter_mut()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(range, device)| device.read(address - range.start()))
            .unwrap_or_else(|| panic!("no byte to be read at address 0x{:04X}", address))
    }

    pub fn write(&mut self, address: u16, data: u8) {
        let address = self.mirror(address);
        self.devices
            .iter_mut()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(range, device)| device.write(address - range.start(), data))
            .unwrap_or_else(|| panic!("can not write to address 0x{:04X}", address))
    }

    fn mirror(&self, address: u16) -> u16 {
        match self
            .mirrors
            .iter()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
        {
            Some((_, max)) => address & max,
            _ => address,
        }
    }
}
