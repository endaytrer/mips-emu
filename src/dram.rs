use crate::{devices::device::Device, bus::DRAM_SIZE, exception::Exception, utils::{concat_halfword, concat_word, get_byte_from_halfword, get_byte_from_word}};

pub struct Dram {
    pub content: [u8; DRAM_SIZE as usize],
}

impl Dram {
    pub fn new() -> Self {
        Self {
            content: [0; DRAM_SIZE as usize],
        }
    }
}
impl Device for Dram {
    fn read(&mut self, addr: u32, size: crate::cpu::Size) -> Result<u32, crate::exception::Exception> {
        if addr as usize >= self.content.len() {
            return Err(Exception::LoadIllegalAddress);
        }
        match size {
            crate::cpu::Size::Byte => {
                Ok(self.content[addr as usize] as u32)
            },
            crate::cpu::Size::Halfword => {
                if addr % 2 != 0 {
                    Err(Exception::LoadIllegalAddress)
                } else {
                    Ok(concat_halfword([self.content[addr as usize], self.content[(addr + 1) as usize]]) as u32)
                }
            },
            crate::cpu::Size::Word => {
                if addr % 4 != 0 {
                    Err(Exception::LoadIllegalAddress)
                } else {
                    Ok(concat_word([self.content[addr as usize], self.content[(addr + 1) as usize], self.content[(addr + 2) as usize], self.content[(addr + 3) as usize]]))
                }
            },
        }
    }

    fn write(&mut self, addr: u32, data: u32, size: crate::cpu::Size) -> Result<(), crate::exception::Exception> {
        if addr as usize >= self.content.len() {
            return Err(Exception::LoadIllegalAddress);
        }
        match size {
            crate::cpu::Size::Byte => {
                self.content[addr as usize] = data as u8;
                Ok(())
            },
            crate::cpu::Size::Halfword => {
                if addr % 2 != 0 {
                    Err(Exception::LoadIllegalAddress)
                } else {
                    self.content[addr as usize] = get_byte_from_halfword(data as u16, 0);
                    self.content[addr as usize + 1] = get_byte_from_halfword(data as u16, 1);
                    Ok(())
                }
            },
            crate::cpu::Size::Word => {
                if addr % 4 != 0 {
                    Err(Exception::LoadIllegalAddress)
                } else {
                    self.content[addr as usize] = get_byte_from_word(data, 0);
                    self.content[addr as usize + 1] = get_byte_from_word(data, 1);
                    self.content[addr as usize + 2] = get_byte_from_word(data, 2);
                    self.content[addr as usize + 3] = get_byte_from_word(data, 3);
                    Ok(())
                }
            },
        }
    }
}