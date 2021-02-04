use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, ops::RangeInclusive};

#[derive(Default)]
pub struct Bus<'a> {
    readable: HashMap<RangeInclusive<u16>, Rc<RefCell<dyn BusRead + 'a>>>,
    writable: HashMap<RangeInclusive<u16>, Rc<RefCell<dyn BusWrite + 'a>>>,
}

pub trait BusRead {
    fn read(&self, address: u16) -> u8;
}

pub trait BusWrite {
    fn write(&mut self, address: u16, data: u8);
}

impl<'a> Bus<'a> {
    pub fn connect<RW: BusRead + BusWrite + 'a>(
        &mut self,
        addressable_range: RangeInclusive<u16>,
        device: &Rc<RefCell<RW>>,
    ) {
        self.readable
            .insert(addressable_range.clone(), Rc::<RefCell<RW>>::clone(device));
        self.writable
            .insert(addressable_range, Rc::<RefCell<RW>>::clone(device));
    }

    pub fn connect_r<R: BusRead + 'a>(
        &mut self,
        addressable_range: RangeInclusive<u16>,
        device: &Rc<RefCell<R>>,
    ) {
        self.readable
            .insert(addressable_range, Rc::<RefCell<R>>::clone(device));
    }

    pub fn connect_w<W: BusWrite + 'a>(
        &mut self,
        addressable_range: RangeInclusive<u16>,
        device: &Rc<RefCell<W>>,
    ) {
        self.writable
            .insert(addressable_range, Rc::<RefCell<W>>::clone(device));
    }

    pub fn read(&self, address: u16) -> u8 {
        self.readable
            .iter()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(range, device)| device.borrow().read(address - range.start()))
            .unwrap_or_else(|| panic!("no byte to be read at address 0x{:04X}", address))
    }

    pub fn write(&mut self, address: u16, data: u8) {
        self.writable
            .iter_mut()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(range, device)| device.borrow_mut().write(address - range.start(), data))
            .unwrap_or_else(|| panic!("can not write to address 0x{:04X}", address))
    }
}
