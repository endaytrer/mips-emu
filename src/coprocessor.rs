
use std::{sync::{Mutex, Arc}, thread, time::Duration};

use crate::{cpu::Size, exception::Exception, devices::device::Device, utils::{set_byte_of_word, set_halfword_of_word, get_byte_from_word, get_halfword_from_word}};
use crate::memory::{PRESENT, READ, VALID, WRITE};

pub const PTBASE: u8 = 4;
pub const COUNT: u8 = 9;
pub const COMPARE: u8 = 11;
pub const SR: u8 = 12;
pub const CAUSE: u8 = 13;
pub const EPC: u8 = 14;
pub const EBASE: u8 = 15;
const TIMER_INTERVAL_MS: u64 = 10;
pub const TIMER_LEVEL: u8 = 5;
pub struct Coprocessor0 {
    pub timer: thread::JoinHandle<()>,
    pub registers: [Arc<Mutex<u32>>; 32],
}

impl Coprocessor0 {
    pub fn new() -> Self {
        let registers = [
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(PRESENT | VALID | READ | WRITE)), // PTBASE
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)), // COUNT
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(10)), // COMPARE
            Arc::new(Mutex::new(0x0000ff01)), // SR
            Arc::new(Mutex::new(0)), // CAUSE
            Arc::new(Mutex::new(0)), // EPC
            Arc::new(Mutex::new(0x80000000)), // EBASE
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0)),
            Arc::new(Mutex::new(0))];
        let count = registers[COUNT as usize].clone();
        let compare = registers[COMPARE as usize].clone();
        let cause = registers[CAUSE as usize].clone();
        Coprocessor0 { registers, timer: thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(TIMER_INTERVAL_MS));
                let mut count_ptr = count.lock().unwrap();
                *count_ptr += 1;
                let compare_ptr = compare.lock().unwrap();
                if *count_ptr == *compare_ptr {
                    *count_ptr = 0;
                    let mut ptr = cause.lock().unwrap();
                    *ptr = (*ptr | (1 << (TIMER_LEVEL + 8))) & 0xffffff83;
                }
            }
        }) }
    }
}
impl Device for Coprocessor0 {
    fn read(&mut self, addr: u32, size: Size) -> Result<u32, Exception> {
        let base = (addr >> 2) as usize;
        let offset = (addr & 0x3) as u8;
        match size {
            Size::Byte => {
                let val = self.registers[base].lock().unwrap();
                Ok(get_byte_from_word(*val, offset) as u32)
            },
            Size::Halfword => match offset {
                0 | 2 => {
                    let val = self.registers[base].lock().unwrap();
                    Ok(get_halfword_from_word(*val, offset) as u32)
                },
                _ => Err(Exception::LoadIllegalAddress)
            }
            Size::Word => match offset {
                0 => {
                    let val = self.registers[base].lock().unwrap();
                    Ok(*val)
                },
                _ => Err(Exception::LoadIllegalAddress)
            }
        }
    }

    fn write(&mut self, addr: u32, data: u32, size: Size) -> Result<(), Exception> {
        let base = (addr >> 2) as usize;
        let offset = (addr & 0x3) as u8;
        match size {
            Size::Byte => {
                let mut val = self.registers[base].lock().unwrap();
                *val = set_byte_of_word(*val, offset, data as u8);
                Ok(())
            },
            Size::Halfword => match offset {
                0 | 2 => {
                    let mut val = self.registers[base].lock().unwrap();
                    *val = set_halfword_of_word(*val, offset, data as u16);
                    Ok(())
                },
                _ => Err(Exception::LoadIllegalAddress)
            }
            Size::Word => match offset {
                0 => {
                    let mut val = self.registers[base].lock().unwrap();
                    *val = data;
                    Ok(())
                },
                _ => Err(Exception::LoadIllegalAddress)
            }
        }
    }
}