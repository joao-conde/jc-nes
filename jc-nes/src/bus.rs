use std::cell::UnsafeCell;
use std::ops::RangeInclusive;
use std::rc::Rc;

#[derive(Default)]
pub struct Bus {
    devices: Vec<(RangeInclusive<u16>, Box<dyn Device>)>,
    mirrors: Vec<(RangeInclusive<u16>, u16)>,
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
        self.devices.push((addressable_range, Box::new(device)));
    }

    pub fn add_mirror(&mut self, addressable_range: RangeInclusive<u16>, max: u16) {
        self.mirrors.push((addressable_range, max));
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

impl Device for Bus {
    fn read(&mut self, address: u16) -> u8 {
        let address = self.mirror(address);
        self.devices
            .iter_mut()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(range, device)| device.read(address - range.start()))
            .unwrap_or_else(|| panic!("no device to read from at address 0x{:04X}", address))
    }

    fn write(&mut self, address: u16, data: u8) {
        let address = self.mirror(address);
        self.devices
            .iter_mut()
            .find(|(addressable_range, _)| addressable_range.contains(&address))
            .map(|(range, device)| device.write(address - range.start(), data))
            .unwrap_or_else(|| panic!("no device to write to at address 0x{:04X}", address))
    }
}

pub type SharedMut<T> = Rc<UnsafeCell<T>>;

pub trait UnsafeDerefMut<T> {
    fn inner(&self) -> &mut T;
}

impl<T> UnsafeDerefMut<T> for SharedMut<T> {
    fn inner(&self) -> &mut T {
        unsafe { &mut *self.get() }
    }
}

impl<T: Device> Device for SharedMut<T> {
    fn read(&mut self, address: u16) -> u8 {
        unsafe { (*self.get()).read(address) }
    }

    fn write(&mut self, address: u16, data: u8) {
        unsafe { (*self.get()).write(address, data) }
    }
}
