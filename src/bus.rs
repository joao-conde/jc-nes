use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, ops::RangeInclusive};

use crate::nes::SharedMut;

#[derive(Default)]
pub struct Bus<'a> {
    devices: HashMap<RangeInclusive<u16>, SharedMut<dyn Device + 'a>>,
    mirrors: HashMap<RangeInclusive<u16>, u16>,
}

pub trait Device {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

impl<'a> Bus<'a> {
    pub fn connect<D: Device + 'a>(
        &mut self,
        addressable_range: RangeInclusive<u16>,
        device: &SharedMut<D>,
    ) {
        self.devices
            .insert(addressable_range, Rc::<RefCell<D>>::clone(device));
    }

    // TODO: change mirroring -> adding a 0x0000-0x1FFF RAM with 0x07FF mirroring is the same as
    // adding 0x0000-0x07FF RAM + 0x07FF-2*0x07FF RAM + ...
    // this prevents having the extra search for mirrors in the begin of read/write
    pub fn add_mirror(&mut self, addressable_range: RangeInclusive<u16>, max: u16) {
        self.mirrors.insert(addressable_range, max);
    }

    pub fn read(&self, address: u16) -> u8 {
        let address = self.mirror(address);
        self.devices
            .iter()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(range, device)| device.borrow_mut().read(address - range.start()))
            .unwrap_or_else(|| panic!("no byte to be read at address 0x{:04X}", address))
    }

    pub fn write(&mut self, address: u16, data: u8) {
        let address = self.mirror(address);
        self.devices
            .iter_mut()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(range, device)| device.borrow_mut().write(address - range.start(), data))
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
