use crate::{cpu::Size, exception::Exception};

pub trait Device {
    fn read(&mut self, addr: u32, size: Size) -> Result<u32, Exception>;
    fn write(&mut self, addr: u32, data: u32, size: Size) -> Result<(), Exception>;
}