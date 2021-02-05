use crate::bus::{BusRead, BusWrite};

pub mod mapper000;

pub trait Mapper: BusRead + BusWrite {}

#[derive(Clone, Copy)]
pub enum MapperMemoryPin {
    ChrROM,
    PrgROM,
}
