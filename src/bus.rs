use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, ops::RangeInclusive};

#[derive(Default)]
pub struct Bus {
    devices: HashMap<RangeInclusive<u16>, Box<dyn Device>>,
    mirrors: HashMap<RangeInclusive<u16>, u16>,
}

impl Bus {
    pub fn connect(
        &mut self,
        addressable_range: RangeInclusive<u16>,
        device: impl Device + 'static,
    ) {
        self.devices.insert(addressable_range, Box::new(device));
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
            .unwrap_or_else(|| panic!("no device to read from at address 0x{:04X}", address))
    }

    pub fn write(&mut self, address: u16, data: u8) {
        let address = self.mirror(address);
        self.devices
            .iter_mut()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(range, device)| device.write(address - range.start(), data))
            .unwrap_or_else(|| panic!("no device to write to at address 0x{:04X}", address))
    }

    fn mirror(&self, address: u16) -> u16 {
        if let Some((_, max)) = self
            .mirrors
            .iter()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
        {
            address & max
        } else {
            address
        }
    }
}

pub trait Device {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

// Interior mutability pattern (use sparingly)
pub type SharedMut<T> = Rc<RefCell<T>>;
impl<T: Device> Device for SharedMut<T> {
    fn read(&mut self, address: u16) -> u8 {
        self.borrow_mut().read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.borrow_mut().write(address, data);
    }
}
