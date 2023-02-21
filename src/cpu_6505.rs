pub mod sr_flags {
    pub const CARRY: u8 = 1;
    pub const ZERO: u8 = 2;
    pub const INTERRUPT: u8 = 4;
    pub const DECIMAL: u8 = 8;
    pub const BREAK: u8 = 16;
    pub const UNUSED: u8 = 32;
    pub const OVERFLOW: u8 = 64;
    pub const NEGATIVE: u8 = 128;
}

pub mod address_space {
    pub const STACK: u16 = 0x0100;
    pub const NMI_VECTOR: u16 = 0xFFFA;
    pub const RES_VECTOR: u16 = 0xFFFC;
    pub const IRQ_VECTOR: u16 = 0xFFFE;
}

pub trait AddressBus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub struct Instruction {
    pub name: Mnemonic,
    pub mode: AddressMode
}

pub enum AddressMode {
    Special,
    Implied,
    Accumulator,
    Immediate,
    Absolute(AccessType),
    ZeroPage(AccessType),
    ZeroPageIndexedX(AccessType),
    ZeroPageIndexedY(AccessType),
    AbsoluteIndexedX(AccessType),
    AbsoluteIndexedY(AccessType),
    Relative,
    XIndexedIndirect(AccessType),
    IndirectIndexedY(AccessType),
    Indirect
}

#[derive(Clone, Copy, Debug)]
pub enum AccessType {
    Read,
    ReadModifyWrite,
    Write
}

#[derive(Debug, Copy, Clone)]
pub enum Mnemonic {
    Brk,
    Ora,
    Asl,
    Php,
    Bpl,
    Clc,
    Jsr,
    And,
    Bit,
    Rol,
    Plp,
    Bmi,
    Sec,
    Rti,
    Eor,
    Lsr,
    Pha,
    Jmp,
    Bvc,
    Cli,
    Rts,
    Adc,
    Ror,
    Pla,
    Bvs,
    Sei,
    Sta,
    Sty,
    Stx,
    Dey,
    Txa,
    Bcc,
    Tya,
    Txs,
    Ldy,
    Lda,
    Ldx,
    Tay,
    Tax,
    Bcs,
    Clv,
    Tsx,
    Cpy,
    Cmp,
    Dec,
    Iny,
    Dex,
    Bne,
    Cld,
    Cpx,
    Sbc,
    Inc,
    Inx,
    Nop,
    Beq,
    Sed,
}