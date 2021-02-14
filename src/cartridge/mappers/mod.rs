use crate::bus::Device;

pub mod mapper000;

pub trait Mapper: Device {}

#[derive(Clone, Copy)]
pub enum MapperMemoryPin {
    ChrROM,
    PrgROM,
}
