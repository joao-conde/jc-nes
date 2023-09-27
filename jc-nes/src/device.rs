use std::{cell::RefCell, rc::Rc};

pub trait Device {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

pub type SharedMut<T> = Rc<RefCell<T>>;

impl<T: Device> Device for SharedMut<T> {
    fn read(&mut self, address: u16) -> u8 {
        self.borrow_mut().read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.borrow_mut().write(address, data);
    }
}

impl<T: Device + ?Sized> Device for Box<T> {
    fn read(&mut self, address: u16) -> u8 {
        self.as_mut().read(address)
    }

    fn write(&mut self, address: u16, data: u8) {
        self.as_mut().write(address, data);
    }
}