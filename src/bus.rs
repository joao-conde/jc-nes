use std::{collections::HashMap, ops::Range};

pub struct Bus {
    pub addresses: HashMap<Range<usize>, Box<dyn Device>>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            addresses: HashMap::new(),
        }
    }

    pub fn connect(&self) {}

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

trait Device {
    fn read(&self, address: usize) -> usize {
        0
    }
    fn write(&self, address: usize) {}
}
