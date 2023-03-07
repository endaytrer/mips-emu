use std::sync::{Arc, Mutex};

use crate::{
    bus::{Bus, COPROCESSOR_BASE},
    exception::Exception,
    devices::device::Device,
    memory,
    utils::sgn_ext_imm_16,
    coprocessor::{SR, EPC, CAUSE, EBASE}
};

pub const REGISTERS_COUNT: usize = 32;
pub const REBOOT_VECTOR: u32 = 0x0;
pub const PGSIZE: u32 = 0x1000;


// register names
pub const ZERO: u8 = 0;
pub const AT: u8   = 1;
pub const V0: u8   = 2;
pub const V1: u8   = 3;
pub const A0: u8   = 4;
pub const A1: u8   = 5;
pub const A2: u8   = 6;
pub const A3: u8   = 7;
pub const T0: u8   = 8;
pub const T1: u8   = 9;
pub const T2: u8   = 10;
pub const T3: u8   = 11;
pub const T4: u8   = 12;
pub const T5: u8   = 13;
pub const T6: u8   = 14;
pub const T7: u8   = 15;
pub const S0: u8   = 16;
pub const S1: u8   = 17;
pub const S2: u8   = 18;
pub const S3: u8   = 19;
pub const S4: u8   = 20;
pub const S5: u8   = 21;
pub const S6: u8   = 22;
pub const S7: u8   = 23;
pub const T8: u8   = 24;
pub const T9: u8   = 25;
pub const K0: u8   = 26;
pub const K1: u8   = 27;
pub const GP: u8   = 28;
pub const SP: u8   = 29;
pub const FP: u8   = 30;
pub const RA: u8   = 31;

#[derive(PartialEq)]
pub enum Size{
    Byte = 8,
    Halfword = 16,
    Word = 32
}

#[derive(Debug)]
pub enum Instruction {
    R {
        opcode: u8,
        rs: u8,
        rt: u8,
        rd: u8,
        shamt: u8,
        funct: u8
    },
    I {
        opcode: u8,
        rs: u8,
        rt: u8,
        imm: u16,
    },
    J {
        opcode: u8,
        imm: u32
    },
    Undefined {
        opcode: u8,
        rest: u32,
    }
}
impl Instruction {
    pub fn new(inst: u32) -> Self {
        let opcode: u8 = (inst >> 26) as u8;
        let rs: u8 = ((inst >> 21) & 0x1f) as u8;
        let rt: u8 = ((inst >> 16) & 0x1f) as u8;
        let rd: u8 = ((inst >> 11) & 0x1f) as u8;
        let shamt: u8 = ((inst >> 6) & 0x1f) as u8;
        let funct: u8 = (inst & 0x3f) as u8;
        let imm16: u16 = (inst & 0xffff) as u16;
        let imm26: u32 = (inst & 0x07ffffff) as u32;
        match opcode {
            0x0 | 0x10 => Self::R { opcode, rs, rt, rd, shamt, funct },
            0x4 | 0x5 | 0x8 | 0x9 | 0xa | 0xb | 0xc | 0xd | 0xf | 0x23 | 0x24 | 0x25 | 0x28 | 0x29 | 0x2b | 0x30 | 0x38 => Self::I { opcode, rs, rt, imm: imm16 },
            0x2 | 0x3 => Self::J { opcode, imm: imm26 },
            _ => Self::Undefined { opcode, rest: imm26 }
        }
    }
    pub fn dump(&self) -> u32 {
        match self {
            Instruction::R { opcode, rs, rt, rd, shamt, funct } => ((opcode.clone() as u32) << 26) |
                    ((rs.clone() as u32) << 21) |
                    ((rt.clone() as u32) << 16) |
                    ((rd.clone() as u32) << 11) |
                    ((shamt.clone() as u32) << 6) |
                    (funct.clone() as u32),
            Instruction::I { opcode, rs, rt, imm } => ((opcode.clone() as u32) << 26) |
                    ((rs.clone() as u32) << 21) |
                    ((rt.clone() as u32) << 16) |
                    (imm.clone() as u32),
            Instruction::J { opcode, imm } => ((opcode.clone() as u32) << 26) |
                    (imm.clone()),
            Instruction::Undefined { opcode, rest } => ((opcode.clone() as u32) << 26) | (rest.clone() as u32),
        }
    }
}

pub struct Cpu {
    registers: [u32; REGISTERS_COUNT],
    pc: u32,
    hi: u32,
    lo: u32,
    pub bus: Bus
}
impl Cpu {
    pub fn new(kernel_file: &str) -> Self {
        let mut bus = Bus::new();
        memory::create_meta_page_table(&mut bus.dram);
        memory::load_kernel(&mut bus.dram, kernel_file);
        Cpu { registers: [0; REGISTERS_COUNT], pc: REBOOT_VECTOR, bus, hi: 0, lo: 0 }
    }
    pub fn load_coprocessor0(&mut self, reg_code: u8) -> Result<u32, Exception> {
        self.bus.read(COPROCESSOR_BASE + ((reg_code as u32) << 2), Size::Word)
    }
    pub fn write_coprocessor0(&mut self, reg_code: u8, data: u32) -> Result<(), Exception> {
        self.bus.write(COPROCESSOR_BASE + ((reg_code as u32) << 2), data, Size::Word)
    }
    fn execute(&mut self) -> Result<u32, Exception> {
        // Fetch
        let ppc = memory::walkpgdir(self, self.pc)?;
        if !ppc.user && ((self.load_coprocessor0(SR)?) >> 4 & 1 != 0) || !ppc.read {
            return Err(Exception::LoadIllegalAddress);
        }
        let res = self.bus.read(ppc.paddr, Size::Word)?;
        // Decode
        let inst = Instruction::new(res);
        // Execute, Memory, WriteBack
        match inst {
            Instruction::R { opcode, rs, rt, rd, shamt, funct } => {
                if opcode == 0 {
                    match funct {
                        0x00 => {
                            // sll;
                            self.registers[rd as usize] = self.registers[rt as usize] << shamt;
                        }
                        0x02 => {
                            // srl
                            self.registers[rd as usize] = self.registers[rt as usize] >> shamt;
                        }
                        0x03 => {
                            // sra
                            self.registers[rd as usize] = self.registers[rt as usize] >> shamt;
                        }
                        0x0c => {
                            // syscall
                            return Err(Exception::Syscall);
                        }
                        0x10 => {
                            // mfhi
                            self.registers[rd as usize] = self.hi;
                        }
                        0x11 => {
                            // mthi
                            self.hi = self.registers[rd as usize];
                        }
                        0x12 => {
                            // mflo
                            self.registers[rd as usize] = self.lo;
                        }
                        0x13 => {
                            // mtlo
                            self.lo = self.registers[rd as usize];
                        }
                        0x18 => {
                            // mult
                            let val = (self.registers[rs as usize] as i32 as i64) * (self.registers[rt as usize] as i32 as i64);
                            self.hi = (val >> 32) as u32;
                            self.lo = (val & 0xffffffff) as u32
                        }
                        0x19 => {
                            // multu
                            let val = (self.registers[rs as usize] as u64) * (self.registers[rt as usize] as u64);
                            self.hi = (val >> 32) as u32;
                            self.lo = (val & 0xffffffff) as u32
                        }
                        0x1a => {
                            // div
                            self.lo = ((self.registers[rs as usize] as i32) / (self.registers[rt as usize] as i32)) as u32;
                            self.hi = ((self.registers[rs as usize] as i32) % (self.registers[rt as usize] as i32)) as u32;
                        }
                        0x1b => {
                            // divu
                            self.lo = (self.registers[rs as usize]) / (self.registers[rt as usize]);
                            self.hi = (self.registers[rs as usize]) % (self.registers[rt as usize]);
                        }
                        0x20 => {
                            // add
                            self.registers[rd as usize] = (self.registers[rs as usize] as i32 + self.registers[rt as usize] as i32) as u32;
                        }
                        0x21 => {
                            // addu
                            self.registers[rd as usize] = self.registers[rs as usize] + self.registers[rt as usize];
                        }
                        0x22 => {
                            // sub
                            self.registers[rd as usize] = (self.registers[rs as usize] as i32 - self.registers[rt as usize] as i32) as u32;
                        }
                        0x23 => {
                            // subu
                            self.registers[rd as usize] = self.registers[rs as usize] - self.registers[rt as usize];
                        }
                        0x24 => {
                            // and
                            self.registers[rd as usize] = self.registers[rs as usize] & self.registers[rt as usize];
                        }
                        0x25 => {
                            // or
                            self.registers[rd as usize] = self.registers[rs as usize] | self.registers[rt as usize];
                        }
                        0x26 => {
                            // xor
                            self.registers[rd as usize] = self.registers[rs as usize] ^ self.registers[rt as usize];
                        }
                        0x27 => {
                            // nor
                            self.registers[rd as usize] = !(self.registers[rs as usize] | self.registers[rt as usize]);
                        }
                        0x2a => {
                            // slt
                            if (self.registers[rs as usize] as i32) < (self.registers[rt as usize] as i32) {
                                self.registers[rd as usize] = 1;
                            } else {
                                self.registers[rd as usize] = 0;
                            }
                        }
                        0x2b => {
                            // sltu
                            if self.registers[rs as usize] < self.registers[rt as usize] {
                                self.registers[rd as usize] = 1;
                            } else {
                                self.registers[rd as usize] = 0;
                            }
                        }
                        _ => {
                            return Err(Exception::InstructionBusError);
                        }
                    }
                } else if opcode == 0x10 {
                    if rs == 0 {
                        // mfc0
                        self.registers[rt as usize] = self.load_coprocessor0(rd)?;
                    } else if rs == 4 {
                        // mtc0
                        self.write_coprocessor0(rd, self.registers[rt as usize])?;
                    } else if rs == 0x10 {
                        // eret
                        let sr = self.load_coprocessor0(SR)?;
                        self.write_coprocessor0(SR, sr & 0xfffffffd)?; // clear exception
                        return Ok(self.load_coprocessor0(EPC)?)
                    } else {
                        return Err(Exception::InstructionBusError)
                    }
                } else {
                    return Err(Exception::InstructionBusError)
                }
            },
            Instruction::I { opcode, rs, rt, imm } => {
                match opcode {
                    0x4 => {
                        // beq
                        if self.registers[rs as usize] == self.registers[rt as usize] {
                            return Ok(((self.pc as i32) + 4 + (sgn_ext_imm_16(imm) << 2)) as u32);
                        }
                    }
                    0x5 => {
                        // bne
                        if self.registers[rs as usize] != self.registers[rt as usize] {
                            return Ok(((self.pc as i32) + 4 + (sgn_ext_imm_16(imm) << 2)) as u32);
                        }
                    }
                    0x8 => {
                        // addi
                        self.registers[rt as usize] = (self.registers[rs as usize] as i32 + sgn_ext_imm_16(imm)) as u32;
                    }
                    0x9 => {
                        // addiu
                        self.registers[rt as usize] = self.registers[rs as usize] + (imm as u32);
                    }
                    0xa => {
                        // slti
                        if (self.registers[rs as usize] as i32) < sgn_ext_imm_16(imm) {
                            self.registers[rt as usize] = 1;
                        } else {
                            self.registers[rt as usize] = 0;
                        }
                    }
                    0xb => {
                        // sltiu
                        if self.registers[rs as usize] < imm as u32 {
                            self.registers[rt as usize] = 1;
                        } else {
                            self.registers[rt as usize] = 0;
                        }
                    }
                    0xc => {
                        // andi
                        self.registers[rt as usize] = self.registers[rs as usize] & (imm as u32);
                    }
                    0xd => {
                        // ori
                        self.registers[rt as usize] = self.registers[rs as usize] | (imm as u32);
                    }
                    0xe => {
                        // xori
                        self.registers[rt as usize] = self.registers[rs as usize] ^ (imm as u32);
                    }
                    0xf => {
                        // lui
                        self.registers[rt as usize] = (imm as u32) << 16;
                    }
                    0x23 => {
                        // lw
                        let paddr = memory::walkpgdir(self, (self.registers[rs as usize] as i32 + sgn_ext_imm_16(imm)) as u32)?;
                        let user = self.load_coprocessor0(SR)? >> 4 & 1 != 0;
                        if user && !paddr.user || !paddr.read  {
                            return Err(Exception::LoadIllegalAddress);
                        }
                        self.registers[rt as usize] = self.bus.read(paddr.paddr, Size::Word)?;
                    }
                    0x24 => {
                        // lbu
                        let paddr = memory::walkpgdir(self, (self.registers[rs as usize] as i32 + sgn_ext_imm_16(imm)) as u32)?;
                        let user = self.load_coprocessor0(SR)? >> 4 & 1 != 0;
                        if user && !paddr.user || !paddr.read  {
                            return Err(Exception::LoadIllegalAddress);
                        }
                        self.registers[rt as usize] = self.bus.read(paddr.paddr, Size::Byte)?;
                    }
                    0x25 => {
                        // lhu
                        let paddr = memory::walkpgdir(self, (self.registers[rs as usize] as i32 + sgn_ext_imm_16(imm)) as u32)?;
                        let user = self.load_coprocessor0(SR)? >> 4 & 1 != 0;
                        if user && !paddr.user || !paddr.read  {
                            return Err(Exception::LoadIllegalAddress);
                        }
                        self.registers[rt as usize] = self.bus.read(paddr.paddr, Size::Halfword)?;
                    }
                    0x28 => {
                        // sb
                        let paddr = memory::walkpgdir(self, (self.registers[rs as usize] as i32 + sgn_ext_imm_16(imm)) as u32)?;
                        let user = self.load_coprocessor0(SR)? >> 4 & 1 != 0;
                        if user && !paddr.user || !paddr.write  {
                            return Err(Exception::StoreIllegalAddress);
                        }
                        self.bus.write(paddr.paddr, self.registers[rt as usize] & 0xff, Size::Byte)?;
                        memory::set_page_dirty(self, (sgn_ext_imm_16(imm) as u32) & 0xfffff000)?;
                    }
                    0x29 => {
                        // sh
                        let paddr = memory::walkpgdir(self, (self.registers[rs as usize] as i32 + sgn_ext_imm_16(imm)) as u32)?;
                        let user = self.load_coprocessor0(SR)? >> 4 & 1 != 0;
                        if user && !paddr.user || !paddr.write  {
                            return Err(Exception::StoreIllegalAddress);
                        }
                        self.bus.write(paddr.paddr, self.registers[rt as usize] & 0xffff, Size::Halfword)?;
                        memory::set_page_dirty(self, (sgn_ext_imm_16(imm) as u32) & 0xfffff000)?;
                    }
                    0x2b => {
                        // sw
                        let paddr = memory::walkpgdir(self, (self.registers[rs as usize] as i32 + sgn_ext_imm_16(imm)) as u32)?;
                        let user = self.load_coprocessor0(SR)? >> 4 & 1 != 0;
                        if user && !paddr.user || !paddr.write  {
                            return Err(Exception::StoreIllegalAddress);
                        }
                        self.bus.write(paddr.paddr, self.registers[rt as usize], Size::Word)?;
                        memory::set_page_dirty(self, (sgn_ext_imm_16(imm) as u32) & 0xfffff000)?;
                    }
                    0x30 => {
                        // ll
                        let paddr = memory::walkpgdir(self, (self.registers[rs as usize] as i32 + sgn_ext_imm_16(imm)) as u32)?;
                        let user = self.load_coprocessor0(SR)? >> 4 & 1 != 0;
                        if user && !paddr.user || !paddr.read  {
                            return Err(Exception::LoadIllegalAddress);
                        }
                        self.registers[rt as usize] = self.bus.read(paddr.paddr, Size::Word)?;
                        self.bus.atomic.insert(paddr.paddr);
                    }
                    0x38 => {
                        // sc
                        let paddr = memory::walkpgdir(self, (self.registers[rs as usize] as i32 + sgn_ext_imm_16(imm)) as u32)?;
                        let user = self.load_coprocessor0(SR)? >> 4 & 1 != 0;
                        if user && !paddr.user || !paddr.write  {
                            return Err(Exception::StoreIllegalAddress);
                        }
                        if self.bus.atomic.contains(&paddr.paddr) {
                            self.bus.write(paddr.paddr, self.registers[rt as usize], Size::Word)?;
                            self.registers[rt as usize] = 1;
                        } else {
                            self.registers[rt as usize] = 0;
                        }
                    }
                    _ => {
                        return Err(Exception::InstructionBusError)
                    }
                }
            },
            Instruction::J { opcode, imm } => {
                match opcode {
                    0x2 => {
                        return Ok((((self.pc as i32) + 4) as u32) & 0xf0000000 | (imm << 2));
                    }
                    0x4 => {
                        self.registers[31] = self.pc + 8;
                        return Ok((((self.pc as i32) + 4) as u32) & 0xf0000000 | (imm << 2));
                    }
                    _ => {
                        return Err(Exception::InstructionBusError)
                    }
                }
            },
            Instruction::Undefined { opcode: _, rest: _ } => {
                return Err(Exception::InstructionBusError);
            },
        }
        Ok(self.pc + 4)
    }
    fn tick_except(&mut self) -> Result<(), Exception> {
        // check if interrupted
        let cause = self.load_coprocessor0(CAUSE)?;
        let pending_interrupts = cause >> 8 & 0xff;
        let exception_code = cause >> 2 & 0x1f;
        let status = self.load_coprocessor0(SR)?;
        let interrupt_enabled = (status & 1 != 0) && (status & 2 == 0);
        let interrupt_mask = status >> 8 & 0xff;
        if interrupt_enabled && (pending_interrupts & interrupt_mask != 0) && exception_code == 0 {
            println!("dealing interrupt");
            // interrupt occurred, transfer to OS
            self.write_coprocessor0(EPC, self.pc)?;  // if interrupt, pc goes back
            self.pc = self.load_coprocessor0(EBASE)?;
        } else if exception_code != 0 {
            println!("dealing exception");
            // exception occurred, transfer to OS
            self.write_coprocessor0(EPC, self.pc + 4)?;
            self.pc = self.load_coprocessor0(EBASE)?;
        } else {
            match self.execute() {
                Ok(pc_dst) => {
                    self.pc = pc_dst;
                }
                Err(exception) => {
                    println!("single error asserted");
                    let sr = self.load_coprocessor0(SR)?;
                    self.write_coprocessor0(SR, sr | 0x2)?;
                    let cause = self.load_coprocessor0(CAUSE)?;
                    self.write_coprocessor0(CAUSE, cause & 0xffffff83 | ((exception as u32) << 2))?;
                }
            };
        }
        Ok(())
    }
    pub fn print_status(&self) {
        println!("Registers");
        for i in 0..32 {
            println!("$r{}: {:#x}", i, self.registers[i]);
        }
        println!("\nPC: {:#x}", self.pc);
    }
    fn tick(&mut self) {
        if let Err(_) = self.tick_except() {
            println!("double error asserted");
            self.pc = REBOOT_VECTOR;
        }
    }
    pub fn interrupt(cause: Arc<Mutex<u32>>, level: u8) {
        let mut ptr = cause.lock().unwrap();
        *ptr = (*ptr | (1 << (level + 8))) & 0xffffff83;
    }
    pub fn run(&mut self, stop_signal: Arc<Mutex<bool>>) {
        while !*(stop_signal.lock().unwrap()) {
            self.tick();
        }
    }
    pub fn debug(&mut self, cycles: usize) {
        for i in 0..cycles {
            self.tick();
            // self.print_status();
        }
    }
}