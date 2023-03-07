use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::coprocessor::{Coprocessor0, CAUSE};
use crate::cpu::Size;
use crate::devices::device::Device;
use crate::dram::Dram;
use crate::exception::Exception;
use crate::rom::Rom;
use crate::devices::{uart::Uart, virtio::Virtio};


pub const DRAM_BASE: u32 = 0x00000000;
pub const DRAM_SIZE: u32 = 0x80000000;
pub const DRAM_END: u32 = DRAM_BASE + DRAM_SIZE - 1;
pub const COPROCESSOR_BASE: u32 = 0xffffc000;
pub const COPROCESSOR_SIZE: u32 = 0x80;
pub const COPROCESSOR_END: u32 = COPROCESSOR_BASE + COPROCESSOR_SIZE - 1;

pub const UART_BASE: u32 = 0xffffd000;
pub const UART_SIZE: u32 = 0x100;
pub const UART_END: u32 = UART_BASE + UART_SIZE - 1;

pub const VIRTIO_BASE: u32 = 0xffffe000;
pub const VIRTIO_SIZE: u32 = 0x1000;
pub const VIRTIO_END: u32 = VIRTIO_BASE + VIRTIO_SIZE - 1;
pub const ROM_BASE: u32 = 0xfffff000;
pub const ROM_SIZE: u32 = 0x1000;
pub const ROM_END: u32 = ROM_BASE - 1 + ROM_SIZE;
pub struct Bus {
    pub dram: Dram,
    pub coprocessor: Coprocessor0,
    uart: Uart,
    pub virtio: Virtio,
    rom: Rom,
    pub atomic: HashSet<u32>
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Rom::new(),
            coprocessor: Coprocessor0::new(),
            uart: Uart::new(),
            virtio: Virtio::new(),
            dram: Dram::new(),
            atomic: HashSet::new()
        }
    }
    pub fn get_raw_cause(&self) -> Arc<Mutex<u32>> {
        self.coprocessor.registers[CAUSE as usize].clone()
    }
    pub fn load_rom(&mut self, file: &str) {
        self.rom.load_binary(file);
    }
}
impl Device for Bus {
    fn read(&mut self, addr: u32, size: Size) -> Result<u32, Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => self.dram.read(addr - DRAM_BASE, size),
            COPROCESSOR_BASE..=COPROCESSOR_END => self.coprocessor.read(addr - COPROCESSOR_BASE, size),
            UART_BASE..=UART_END => self.uart.read(addr - UART_BASE, size),
            VIRTIO_BASE..=VIRTIO_END => self.virtio.read(addr - VIRTIO_BASE, size),
            ROM_BASE..=ROM_END => self.rom.read(addr - ROM_BASE, size),
            _ => Err(Exception::LoadIllegalAddress)
        }
    }
    fn write(&mut self, addr: u32, data: u32, size: Size) -> Result<(), Exception> {
        self.atomic.remove(&addr);
        match addr {
            DRAM_BASE..=DRAM_END => self.dram.write(addr - DRAM_BASE, data, size),
            COPROCESSOR_BASE..=COPROCESSOR_END => self.coprocessor.write(addr - COPROCESSOR_BASE, data, size),
            UART_BASE..=UART_END => self.uart.write(addr - UART_BASE, data, size),
            VIRTIO_BASE..=VIRTIO_END => self.virtio.write(addr - VIRTIO_BASE, data, size),
            ROM_BASE..=ROM_END => self.rom.write(addr - ROM_BASE, data, size),
            _ => Err(Exception::LoadIllegalAddress)
        }
    } 
}