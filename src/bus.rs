use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, ops::RangeInclusive};

#[derive(Default)]
pub struct Bus<'a> {
    pub readable: HashMap<RangeInclusive<u16>, Rc<RefCell<dyn Read + 'a>>>,
    pub writable: HashMap<RangeInclusive<u16>, Rc<RefCell<dyn Write + 'a>>>,
}

pub trait Read {
    fn read(&self, address: u16) -> u8;
}

pub trait Write {
    fn write(&mut self, address: u16, data: u8);
}

impl<'a> Bus<'a> {
    pub fn connect<RW: Read + Write + 'a>(&mut self, addressable_range: RangeInclusive<u16>, device: &Rc<RefCell<RW>>) {
        self.readable.insert(addressable_range.clone(), Rc::<RefCell<RW>>::clone(device));
        self.writable.insert(addressable_range, Rc::<RefCell<RW>>::clone(device));
    }

    pub fn connect_r<R: Read + 'a>(&mut self, addressable_range: RangeInclusive<u16>, device: &Rc<RefCell<R>>) {
        self.readable.insert(addressable_range, Rc::<RefCell<R>>::clone(device));
    }

    pub fn connect_w<W: Write + 'a>(&mut self, addressable_range: RangeInclusive<u16>, device: &Rc<RefCell<W>>) {
        self.writable.insert(addressable_range, Rc::<RefCell<W>>::clone(device));
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        let device = self
            .readable
            .iter()
            .filter(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(_, device)| device)
            .next();

        device.map(|device| device.borrow().read(address))
    }

    pub fn write(&mut self, address: u16, data: u8) -> Option<()> {
        let device = self
            .writable
            .iter_mut()
            .filter(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(_, device)| device)
            .next();

        device.map(|device| device.borrow_mut().write(address, data))
    }
}
