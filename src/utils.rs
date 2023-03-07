use crate::cpu::{Instruction};

fn get_byte_from_halfword_small_endian(src: u16, offset: u8) -> u8 {
    (src >> (offset << 3) & 0xff) as u8
}
fn get_byte_from_word_small_endian(src: u32, offset: u8) -> u8 {
    (src >> (offset << 3) & 0xff) as u8
}
fn get_halfword_from_word_small_endian(src: u32, offset: u8) -> u16 {
    (src >> (offset << 3) & 0xffff) as u16
}

fn get_byte_from_halfword_big_endian(src: u16, offset: u8) -> u8 {
    (src >> (8 - (offset << 3)) & 0xff) as u8
}
fn get_byte_from_word_big_endian(src: u32, offset: u8) -> u8 {
    (src >> (24 - (offset << 3)) & 0xff) as u8
}
fn get_halfword_from_word_big_endian(src: u32, offset: u8) -> u16 {
    (src >> (24 - (offset << 3)) & 0xffff) as u16
}


fn set_byte_of_halfword_small_endian(src: u16, offset: u8, data: u8) -> u16 {
    src & !(0xff << (offset << 3)) | ((data as u16) << (offset << 3))
}
fn set_byte_of_word_small_endian(src: u32, offset: u8, data: u8) -> u32 {
    src & !(0xff << (offset << 3)) | ((data as u32) << (offset << 3))
}
fn set_halfword_of_word_small_endian(src: u32, offset: u8, data: u16) -> u32 {
    src & !(0xffff << (offset << 3)) | ((data as u32) << (offset << 3))
}

fn set_byte_of_halfword_big_endian(src: u16, offset: u8, data: u8) -> u16 {
    src & !(0xff << (8 - (offset << 3))) | ((data as u16) << (8 - (offset << 3)))
}
fn set_byte_of_word_big_endian(src: u32, offset: u8, data: u8) -> u32 {
    src & !(0xff << (24 - (offset << 3))) | ((data as u32) << (24 - (offset << 3)))
}
fn set_halfword_of_word_big_endian(src: u32, offset: u8, data: u16) -> u32 {
    src & !(0xffff << (24 - (offset << 3))) | ((data as u32) << (24 - (offset << 3)))
}
fn concat_halfword_small_endian(src: [u8; 2]) -> u16 {
    (src[0] as u16) | ((src[1] as u16) << 8)
}
fn concat_word_small_endian(src: [u8; 4]) -> u32 {
    (src[0] as u32) | ((src[1] as u32) << 8) | ((src[2] as u32) << 16) | ((src[3] as u32) << 24)
}
fn concat_halfword_big_endian(src: [u8; 2]) -> u16 {
    (src[1] as u16) | ((src[0] as u16) << 8)
}
fn concat_word_big_endian(src: [u8; 4]) -> u32 {
    (src[3] as u32) | ((src[2] as u32) << 8) | ((src[1] as u32) << 16) | ((src[0] as u32) << 24)
}
// set default small endian
pub const get_byte_from_halfword: fn(src: u16, offset: u8) -> u8            = get_byte_from_halfword_small_endian;
pub const get_byte_from_word: fn(src: u32, offset: u8) -> u8                = get_byte_from_word_small_endian;
pub const get_halfword_from_word: fn(src: u32, offset: u8) -> u16           = get_halfword_from_word_small_endian;
pub const set_byte_of_halfword: fn(src: u16, offset: u8, data: u8) -> u16   = set_byte_of_halfword_small_endian;
pub const set_byte_of_word: fn(src: u32, offset: u8, data: u8) -> u32       = set_byte_of_word_small_endian;
pub const set_halfword_of_word: fn(src: u32, offset: u8, data: u16) -> u32  = set_halfword_of_word_small_endian;
pub const concat_halfword: fn(src: [u8; 2]) -> u16                          = concat_halfword_small_endian;
pub const concat_word: fn(src: [u8; 4]) -> u32                              = concat_word_small_endian;

impl Instruction {
    // R types
    pub fn sll(rd: u8, rt: u8, shamt: u8) -> Self {
        Self::R { opcode: 0x0, rs: 0, rt, rd, shamt, funct: 0x00 }
    }
    pub fn srl(rd: u8, rt: u8, shamt: u8) -> Self {
        Self::R { opcode: 0x0, rs: 0, rt, rd, shamt, funct: 0x02 }
    }
    pub fn sra(rd: u8, rt: u8, shamt: u8) -> Self {
        Self::R { opcode: 0x0, rs: 0, rt, rd, shamt, funct: 0x03 }
    }
    pub fn syscall() -> Self {
        Self::R { opcode: 0x0, rs: 0, rt: 0, rd: 0, shamt: 0, funct: 0xc }
    }
    pub fn mfhi(rd: u8) -> Self {
        Self::R { opcode: 0x0, rs: 0, rt: 0, rd, shamt: 0, funct: 0x10 }
    }
    pub fn mthi(rd: u8) -> Self {
        Self::R { opcode: 0x0, rs: 0x8, rt: 0, rd, shamt: 0, funct: 0x11 }
    }
    pub fn mflo(rd: u8) -> Self {
        Self::R { opcode: 0x0, rs: 0, rt: 0, rd, shamt: 0, funct: 0x12 }
    }
    pub fn mtlo(rd: u8) -> Self {
        Self::R { opcode: 0x0, rs: 0x8, rt: 0, rd, shamt: 0, funct: 0x13 }
    }
    pub fn mult(rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd: 0x0, shamt: 0, funct: 0x18 }
    }
    pub fn multu(rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd: 0x0, shamt: 0, funct: 0x19 }
    }
    pub fn div(rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd: 0, shamt: 0, funct: 0x1a }
    }
    pub fn divu(rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd: 0, shamt: 0, funct: 0x1b }
    }

    pub fn add(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x20 }
    }
    pub fn addu(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x21 }
    }
    pub fn sub(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x22 }
    }
    pub fn subu(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x23 }
    }
    pub fn and(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x24 }
    }
    pub fn or(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x25 }
    }
    pub fn xor(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x26 }
    }
    pub fn nor(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x27 }
    }
    pub fn slt(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x2a }
    }
    pub fn sltu(rd: u8, rs: u8, rt: u8) -> Self {
        Self::R { opcode: 0x0, rs, rt, rd, shamt: 0, funct: 0x2b }
    }

    pub fn mfc0(rt: u8, rd: u8) -> Self {
        Self::R { opcode: 0x10, rs: 0, rt, rd, shamt: 0, funct: 0 }
    }
    pub fn mtc0(rt: u8, rd: u8) -> Self {
        Self::R { opcode: 0x10, rs: 0x4, rt, rd, shamt: 0, funct: 0 }
    }
    pub fn eret() -> Self {
        Self::R { opcode: 0x10, rs: 0x10, rt: 0, rd: 0, shamt: 0, funct: 0x18 }
    }

    // I types
    pub fn beq(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x4, rs, rt, imm }
    }
    pub fn bne(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x5, rs, rt, imm }
    }
    pub fn addi(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x8, rs, rt, imm }
    }
    pub fn addiu(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x9, rs, rt, imm }
    }
    pub fn slti(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0xa, rs, rt, imm }
    }
    pub fn sltiu(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0xb, rs, rt, imm }
    }
    pub fn andi(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0xc, rs, rt, imm }
    }
    pub fn ori(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0xd, rs, rt, imm }
    }
    pub fn xori(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0xe, rs, rt, imm }
    }
    pub fn lui(rt: u8, imm: u16) -> Self {
        Self::I { opcode: 0xf, rs: 0, rt, imm }
    }
    pub fn lw(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x23, rs, rt, imm }
    }
    pub fn lbu(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x24, rs, rt, imm }
    }
    pub fn lhu(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x25, rs, rt, imm }
    }
    pub fn sb(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x28, rs, rt, imm }
    }
    pub fn sh(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x29, rs, rt, imm }
    }
    pub fn sw(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x2b, rs, rt, imm }
    }
    pub fn ll(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x30, rs, rt, imm }
    }
    pub fn sc(rt: u8, rs: u8, imm: u16) -> Self {
        Self::I { opcode: 0x38, rs, rt, imm }
    }

    // J types
    pub fn j(imm: u32) -> Self {
        Self::J { opcode: 0x2, imm }
    }
    pub fn jal(imm: u32) -> Self {
        Self::J { opcode: 0x3, imm }
    }
}
pub fn sgn_ext_imm_16(imm: u16) -> i32 {
    imm as i16 as i32
}