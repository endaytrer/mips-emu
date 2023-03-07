use std::fs;
use elf::{ElfBytes, endian::{AnyEndian, BigEndian, EndianParse}};

use crate::{cpu::{Cpu, Size}, exception::Exception, devices::device::Device, coprocessor::{PTBASE}, bus::ROM_BASE};
use crate::bus::{UART_BASE, VIRTIO_BASE};
use crate::dram::Dram;

pub const HUGE: u32 = 0x40;
pub const PRESENT: u32 = 0x20;
pub const VALID: u32 = 0x10;
pub const USER: u32 = 0x8;
pub const READ: u32 = 0x4;
pub const WRITE: u32 = 0x2;
pub const DIRTY: u32 = 0x1;
// virtual memory allocation
pub const TEXT: u32 = 0x400000;
pub const DATA: u32 = 0x10000000;
pub const HEAP: u32 = 0x10008000;
struct PTE {
    entry: u32
}
impl PTE {
    pub fn pfn(&self) -> u32 {
        self.entry & 0xfffff000
    }
    pub fn huge(&self) -> bool  { (self.entry >> 6) & 1 != 0 }
    pub fn present(&self) -> bool {
        (self.entry >> 5) & 1 != 0
    }
    pub fn valid(&self) -> bool {
        (self.entry >> 4) & 1 != 0
    }
    pub fn user(&self) -> bool {
        (self.entry >> 3) & 1 != 0
    }
    pub fn read(&self) -> bool {
        (self.entry >> 2) & 1 != 0
    }
    pub fn write(&self) -> bool {
        (self.entry >> 1) & 1 != 0
    }
    pub fn dirty(&self) -> bool {
        self.entry & 1 != 0
    }
    
}
pub struct Paddr {
    pub paddr: u32,
    pub user: bool,
    pub read: bool,
    pub write: bool
}
pub fn walkpgdir(cpu: &mut Cpu, vaddr: u32) -> Result<Paddr, Exception> {
    let mut pte = PTE{entry: cpu.load_coprocessor0(PTBASE)?};
    let mut user = true;
    let mut read = true;
    let mut write = true;
    for bit_shift in [22, 12] {
        if !pte.valid() {
            return Err(Exception::LoadIllegalAddress)
        }
        if !pte.present() {
            return Err(Exception::PageFault)
        }
        let offset = ((vaddr >> bit_shift) & 0x3ff) << 2;
        pte = PTE{entry: cpu.bus.read(pte.pfn() | offset, Size::Word)?};
        user &= pte.user();
        read &= pte.read();
        write &= pte.write();
        if pte.huge() && bit_shift == 22 {
            return Ok(Paddr {
                paddr: pte.entry | (vaddr & 0x3fffff),
                user,
                read,
                write,
            })
        }
    }
    Ok(Paddr {
        paddr: pte.pfn() | (vaddr & 0xfff),
        user,
        read,
        write
    })
}
pub fn set_page_dirty(cpu: &mut Cpu, vaddr: u32) -> Result<(), Exception> {
    let mut pte = PTE{entry: cpu.load_coprocessor0(PTBASE)?};
    pte.entry |= 1;
    cpu.write_coprocessor0(PTBASE, pte.entry)?;
    for bit_shift in [22, 12] {
        let offset = ((vaddr >> bit_shift) & 0x3ff) << 2;
        let mut new_pte = PTE{entry: cpu.bus.read(pte.pfn() | offset, Size::Word)?};
        new_pte.entry |= 1;
        cpu.bus.write(pte.pfn() | offset, new_pte.entry, Size::Word)?;
        if new_pte.huge() {
            return Ok(())
        }
        pte = new_pte;
    }
    Ok(())
}
struct Allocator {
    allocated: u32
}
impl Allocator {
    pub fn new() -> Self {
        Self {
            allocated: 2
        }
    }
    pub fn kalloc(&mut self, dram: &mut Dram, vaddr: u32) -> u32 {
        // assume vaddr is page-aligned
        assert_eq!(vaddr & 0xfff, 0);
        let offset = ((vaddr >> 22) & 0x3ff) << 2;
        let mut pde = PTE {entry: dram.read(offset, Size::Word).unwrap()};
        if pde.valid() {
            if pde.huge() {
                panic!("Virtual address allocated");
            } else {
                dram.write(offset, pde.entry | DIRTY, Size::Word).unwrap();
            }
        } else {
            // allocate page table
            let new_pde = (self.allocated << 12) | PRESENT | VALID | READ | WRITE;
            dram.write(offset, new_pde, Size::Word).unwrap();
            self.allocated += 1;
            pde = PTE{entry: new_pde};
        }
        let offset = ((vaddr >> 12) & 0x3ff) << 2;
        let pte = PTE{entry: dram.read(pde.pfn() | offset, Size::Word).unwrap()};
        if pte.valid() {
            panic!("Virtual address allocated");
        }
        let new_pfn = self.allocated << 12;
        let new_pte = new_pfn | PRESENT | VALID | READ | WRITE;
        dram.write(pde.pfn() | offset, new_pte, Size::Word).unwrap();
        self.allocated += 1;
        new_pfn
    }
}
pub fn create_meta_page_table(dram: &mut Dram) {
    dram.write(0, 0x00001000 | PRESENT | VALID | READ | WRITE, Size::Word).unwrap();
    dram.write(0x00001000, ROM_BASE | PRESENT | VALID | READ, Size::Word).unwrap(); // map uart
    dram.write(0x00001004, VIRTIO_BASE | PRESENT | VALID | READ | WRITE, Size::Word).unwrap();
    dram.write(0x00001008, UART_BASE | PRESENT | VALID | READ | WRITE, Size::Word).unwrap();
    // identity mapping from 0x80000000 to 0x00000000
    for i in 0x200..0x400 {
        let addr = i << 2;
        dram.write(addr, ((i - 0x200) << 22) | HUGE | PRESENT | VALID | READ | WRITE, Size::Word).unwrap();
    }
}
pub fn load_kernel(dram: &mut Dram, filename: &str) {
    let mut allocator = Allocator::new();
    let buf = fs::read(filename).unwrap();
    let slice = buf.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).unwrap();
    // .text and .data segment
    let text_header = file.section_header_by_name(".text").unwrap().expect("no .text section");
    let (text_segment, _) = file.section_data(&text_header).unwrap();
    let mut pbase = 0;
    let mut ptr: usize = 0;
    while ptr < text_segment.len() {
        let vaddr = TEXT + ptr as u32;
        if ptr & 0xfff == 0 {
            // page start, allocate page;
            pbase = allocator.kalloc(dram, vaddr);
        }
        let inst = BigEndian.parse_u32_at(&mut ptr, text_segment).unwrap();
        let paddr = pbase + ptr as u32;
        dram.write(paddr, inst, Size::Word).unwrap();
        ptr += 4;
    }

    if let Some(data_header) = file.section_header_by_name(".data").unwrap() {
        let (data_segment, _) = file.section_data(&data_header).unwrap();
        let mut pbase = 0;
        let mut ptr: usize = 0;
        while ptr < data_segment.len() {
            let vaddr = DATA + ptr as u32;
            if ptr & 0xfff == 0 {
                // page start, allocate page;
                pbase = allocator.kalloc(dram, vaddr);
            }
            let inst = BigEndian.parse_u32_at(&mut ptr, data_segment).unwrap();
            let paddr = pbase + ptr as u32;
            dram.write(paddr, inst, Size::Word).unwrap();
            ptr += 4;
        }

    }
}