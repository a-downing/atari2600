use crate::{AddressBus, Mnemonic, sr_flags, address_space, Instruction, AddressMode, AccessType, decode};

pub struct MOS6502<T: AddressBus> {
    pc: u16,
    sr: u8,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    tmp: u8,
    cycle: u8,
    cycles: u32,
    addr: u16,
    addr_invalid: bool,
    ptr: u16,
    ptr_invalid: bool,
    instruction: Instruction,
    bus: T
}

impl<T: AddressBus> MOS6502<T> {
    pub fn new(bus: T) -> Self {
        MOS6502 {
            pc: 0,
            sr: 0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            tmp: 0,
            cycle: 1,
            cycles: 0,
            addr: 0,
            addr_invalid: false,
            ptr: 0,
            ptr_invalid: false,
            instruction: Instruction { name: Mnemonic::Brk, mode: AddressMode::Implied },
            bus
        }
    }

    pub fn get_bus(&mut self) -> &mut T {
        &mut self.bus
    }

    fn push(&mut self, value: u8) {
        self.bus.write(address_space::STACK + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let result = self.bus.read(address_space::STACK + self.sp as u16);
        result
    }

    pub fn reset(&mut self) {
        let lo = self.bus.read(address_space::RES_VECTOR) as u16;
        let hi = self.bus.read(address_space::RES_VECTOR + 1) as u16;
        self.pc = hi << 8 | lo;
    }

    pub fn cycle(&mut self, rdy: bool) {
        match self.cycle {
            1 => {
                if !rdy {
                    return;
                }

                let opcode = self.bus.read(self.pc);
                //println!("0x{:04X}: opcode: 0x{:02X} {:?}, cycles: {}", self.pc, opcode, cpu_6502_decode::decode(opcode), self.cycles);
                //println!("A: 0x{:02X}, X: 0x{:02X}, Y: 0x{:02X}, SP: 0x{:02X}, SR: 0x{:02X}", self.a, self.x, self.y, self.sp, self.sr);
                self.instruction = decode(opcode);
                self.pc += 1;
            },
            _ => match &self.instruction.mode {
                AddressMode::Special => self.special(),
                AddressMode::Implied => self.implied(),
                AddressMode::Accumulator => self.accumulator(),
                AddressMode::Immediate => self.immediate(),
                AddressMode::Absolute(access) => self.absolute(*access),
                AddressMode::ZeroPage(access) => self.zeropage(*access),
                AddressMode::ZeroPageIndexedX(access) => self.zeropage_indexed(self.x, *access),
                AddressMode::ZeroPageIndexedY(access) => self.zeropage_indexed(self.y, *access),
                AddressMode::AbsoluteIndexedX(access) => self.absolute_indexed(self.x, *access),
                AddressMode::AbsoluteIndexedY(access) => self.absolute_indexed(self.y, *access),
                AddressMode::Relative => self.relative(),
                AddressMode::XIndexedIndirect(access) => self.x_indexed_indirect(*access),
                AddressMode::IndirectIndexedY(access) => self.indirect_indexed_y(*access),
                AddressMode::Indirect => match self.instruction.name {
                    Mnemonic::Jmp => match self.cycle {
                        2 => {
                            self.addr = self.bus.read(self.pc) as u16;
                            self.pc += 1;
                        }
                        3 => {
                            self.addr |= (self.bus.read(self.pc) as u16) << 8;
                            self.pc += 1;
                        }
                        4 => self.pc = self.bus.read(self.addr) as u16,
                        5 => {
                            self.cycle = 0;
                            self.pc |= (self.bus.read(self.addr & 0xFF00 | (self.addr as u8).wrapping_add(1) as u16) as u16) << 8; // page crossing not handled
                            //self.pc |= (self.bus.read(self.addr + 1) as u16) << 8;
                        }
                        _ => panic!()
                    }
                    _ => panic!()
                }
            }
        }

        self.cycle += 1;
        self.cycles += 1;
    }

    fn update_flags(&mut self, reg: u8) {
        if reg == 0 {
            self.sr |= sr_flags::ZERO;
        } else {
            self.sr &= !sr_flags::ZERO;
        }

        if reg & 0x80 != 0 {
            self.sr |= sr_flags::NEGATIVE;
        } else {
            self.sr &= !sr_flags::NEGATIVE;
        }
    }

    fn x_indexed_indirect(&mut self, access: AccessType) {
        match access {
            AccessType::Read | AccessType::Write => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.bus.read(self.addr); //QUIRK
                    self.addr = (self.addr as u8).wrapping_add(self.x) as u16
                }
                4 => {
                    self.ptr = self.bus.read(self.addr) as u16;
                }
                5 => {
                    let hi = self.bus.read((self.addr as u8).wrapping_add(1) as u16) as u16;
                    self.ptr |= hi << 8;
                }
                6 => {
                    self.cycle = 0;

                    match access {
                        AccessType::Read => {
                            let value = self.bus.read(self.ptr);
                            self.execute_read(value)
                        },
                        AccessType::Write => {
                            self.execute_write(self.ptr);
                        },
                        _ => panic!()
                    }
                }
                _ => panic!()
            }
            AccessType::ReadModifyWrite => todo!(),
        }
    }

    fn indirect_indexed_y(&mut self, access: AccessType) {
        match access {
            AccessType::Read => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.ptr = self.bus.read(self.addr) as u16;
                }
                4 => {
                    let hi = self.bus.read((self.addr as u8).wrapping_add(1) as u16);
                    self.set_ptr_hi_index_lo(hi, self.y);
                }
                5 => {
                    let value = self.bus.read(self.ptr); // QUIRK
                    self.fix_ptr();
    
                    if !self.ptr_invalid {
                        self.cycle = 0;
                        self.execute_read(value);
                    }
                }
                6 => {
                    self.cycle = 0;
                    let value = self.bus.read(self.ptr);
                    self.execute_read(value)
                }
                _ => panic!()
            }
            AccessType::ReadModifyWrite => todo!(),
            AccessType::Write => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.ptr = self.bus.read(self.addr) as u16;
                }
                4 => {
                    let hi = self.bus.read((self.addr as u8).wrapping_add(1) as u16);
                    self.set_ptr_hi_index_lo(hi, self.y);
                }
                5 => {
                    self.bus.read(self.ptr); // QUIRK
                    self.fix_ptr();
                }
                6 => {
                    self.cycle = 0;
                    self.execute_write(self.ptr);
                }
                _ => panic!()
            },
        }
    }

    fn absolute_indexed(&mut self, i: u8, access: AccessType) {
        match access {
            AccessType::Read => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1
                }
                3 => {
                    let hi = self.bus.read(self.pc);
                    self.set_addr_hi_index_lo(hi, i);
                    self.pc += 1
                }
                4 => {
                    let value = self.bus.read(self.addr); // QUIRK
                    self.fix_addr();

                    if !self.addr_invalid {
                        self.cycle = 0;
                        self.execute_read(value);
                    }
                }
                5 => {
                    self.cycle = 0;
                    let value = self.bus.read(self.addr);
                    self.execute_read(value);
                }
                _ => panic!()
            },
            AccessType::ReadModifyWrite => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    let hi = self.bus.read(self.pc);
                    self.set_addr_hi_index_lo(hi, i);
                    self.pc += 1;
                }
                4 => {
                    self.bus.read(self.addr); // QUIRK
                    self.fix_addr();
                }
                5 => {
                    self.tmp = self.bus.read(self.addr);
                }
                6 => {
                    self.bus.write(self.addr, self.tmp); // QUIRK
                    self.tmp = self.execute_rmw(self.tmp);
                }
                7 => {
                    self.cycle = 0;
                    self.bus.write(self.addr, self.tmp);
                }
                _ => panic!()
            },
            AccessType::Write => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    let hi = self.bus.read(self.pc);
                    self.set_addr_hi_index_lo(hi, i);
                    self.pc += 1;
                }
                4 => {
                    self.bus.read(self.addr); // QUIRK
                    self.fix_addr();
                }
                5 => {
                    self.cycle = 0;
                    self.execute_write(self.addr);
                }
                _ => panic!()
            }
        }
    }

    fn accumulator(&mut self) {
        let _ = self.bus.read(self.pc);
        self.cycle = 0;
        self.a = self.execute_rmw(self.a);
    }

    fn zeropage(&mut self, access: AccessType) {
        match access {
            AccessType::Read => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.cycle = 0;
                    let value = self.bus.read(self.addr);
                    self.execute_read(value);
                }
                _ => panic!()
            },
            AccessType::ReadModifyWrite => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.tmp = self.bus.read(self.addr);
                }
                4 => {
                    self.bus.write(self.addr, self.tmp); // QUIRK
                    self.tmp = self.execute_rmw(self.tmp);
                }
                5 => {
                    self.cycle = 0;
                    self.bus.write(self.addr, self.tmp);
                }
                _ => panic!()
            },
            AccessType::Write => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.cycle = 0;
                    self.execute_write(self.addr);
                }
                _ => panic!()
            }
        }
    }

    fn execute_rmw(&mut self, value: u8) -> u8 {
        match self.instruction.name {
            Mnemonic::Asl => {
                if value & 0x80 != 0 {
                    self.sr |= sr_flags::CARRY;
                } else {
                    self.sr &= !sr_flags::CARRY;
                }

                let result = value << 1;
                self.update_flags(result);
                result
            }
            Mnemonic::Lsr => {
                if value & 0x1 != 0 {
                    self.sr |= sr_flags::CARRY;
                } else {
                    self.sr &= !sr_flags::CARRY;
                }

                let result = value >> 1;
                self.update_flags(result);
                result
            }
            Mnemonic::Rol => {
                let orig = value;
                let result = value << 1 | (self.sr & sr_flags::CARRY);

                if orig & 0b10000000 != 0 {
                    self.sr |= sr_flags::CARRY;
                } else {
                    self.sr &= !sr_flags::CARRY;
                }

                self.update_flags(result);
                result
            }
            Mnemonic::Ror => {
                let orig = value;
                let result = value >> 1 | ((self.sr & sr_flags::CARRY) << 7);

                if orig & 0x1 != 0 {
                    self.sr |= sr_flags::CARRY;
                } else {
                    self.sr &= !sr_flags::CARRY;
                }

                self.update_flags(result);
                result
            }
            Mnemonic::Inc => {
                let result = value.wrapping_add(1);
                self.update_flags(result);
                result
            }
            Mnemonic::Dec => {
                let result = value.wrapping_sub(1);
                self.update_flags(result);
                result
            }
            _ => panic!()
        }
    }

    fn special(&mut self) {
        match self.instruction.name {
            Mnemonic::Jsr => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => (),
                4 => self.push((self.pc >> 8) as u8),
                5 => self.push((self.pc & 0xFF) as u8),
                6 => {
                    self.cycle = 0;
                    self.pc = (self.bus.read(self.pc) as u16) << 8 | self.addr;
                }
                _ => panic!()
            }
            Mnemonic::Rts => match self.cycle {
                2 => { self.bus.read(self.pc); },
                3 => (),
                4 => self.pc = self.pop() as u16,
                5 => self.pc |= (self.pop() as u16) << 8,
                6 => {
                    self.cycle = 0;
                    self.pc += 1
                }
                _ => panic!()
            }
            Mnemonic::Brk => {
                match self.cycle {
                    2 => self.pc += 1,
                    3 => self.push((self.pc >> 8) as u8),
                    4 => self.push((self.pc & 0xFF) as u8),
                    5 => {
                        self.push(self.sr | sr_flags::BREAK | sr_flags::UNUSED);
                        self.sr |= sr_flags::INTERRUPT;
                    }
                    6 => self.pc = self.bus.read(address_space::IRQ_VECTOR) as u16,
                    7 => {
                        self.cycle = 0;
                        self.pc |= (self.bus.read(address_space::IRQ_VECTOR + 1) as u16) << 8;
                    }
                    _ => panic!()
                }
            }
            Mnemonic::Rti => match self.cycle {
                2 => (),
                3 => (),
                4 => self.sr = self.pop() & !sr_flags::BREAK & !sr_flags::UNUSED,
                5 => self.pc = self.pop() as u16,
                6 => {
                    self.cycle = 0;
                    self.pc |= (self.pop() as u16) << 8;
                }
                _ => panic!()
            }
            Mnemonic::Php => match self.cycle {
                2 => { self.bus.read(self.pc); }
                3 => {
                    self.cycle = 0;
                    self.push(self.sr | sr_flags::BREAK | sr_flags::UNUSED);
                }
                _ => panic!()
            }
            Mnemonic::Pha => match self.cycle {
                2 => { self.bus.read(self.pc); }
                3 => {
                    self.cycle = 0;
                    self.push(self.a);
                }
                _ => panic!()
            }
            Mnemonic::Pla => match self.cycle {
                2 => { self.bus.read(self.pc); }
                3 => (),
                4 => {
                    self.cycle = 0;
                    self.a = self.pop();
                    self.update_flags(self.a);
                }
                _ => panic!()
            }
            Mnemonic::Plp => match self.cycle {
                2 => { self.bus.read(self.pc); }
                3 => (),
                4 => {
                    self.cycle = 0;
                    self.sr = self.pop() & !sr_flags::BREAK & !sr_flags::UNUSED
                }
                _ => panic!()
            }
            _ => panic!()
        }
    }

    fn relative(&mut self) {
        match self.cycle {
            2 => {
                let rel = self.bus.read(self.pc) as i8 as i16;
                self.pc += 1;
                self.addr = self.pc.wrapping_add_signed(rel);

                match self.instruction.name {
                    Mnemonic::Bne =>  {
                        if self.sr & sr_flags::ZERO != 0 {
                            self.cycle = 0;
                        }
                    }
                    Mnemonic::Beq =>  {
                        if self.sr & sr_flags::ZERO == 0 {
                            self.cycle = 0;
                        }
                    }
                    Mnemonic::Bpl =>  {
                        if self.sr & sr_flags::NEGATIVE != 0 {
                            self.cycle = 0;
                        }
                    }
                    Mnemonic::Bcs =>  {
                        if self.sr & sr_flags::CARRY == 0 {
                            self.cycle = 0;
                        }
                    }
                    Mnemonic::Bcc =>  {
                        if self.sr & sr_flags::CARRY != 0 {
                            self.cycle = 0;
                        }
                    }
                    Mnemonic::Bmi =>  {
                        if self.sr & sr_flags::NEGATIVE == 0 {
                            self.cycle = 0;
                        }
                    }
                    Mnemonic::Bvc =>  {
                        if self.sr & sr_flags::OVERFLOW != 0 {
                            self.cycle = 0;
                        }
                    }
                    Mnemonic::Bvs =>  {
                        if self.sr & sr_flags::OVERFLOW == 0 {
                            self.cycle = 0;
                        }
                    }
                    _ => panic!()
                }
            }
            3 => {
                if self.addr & 0xFF00 == self.pc & 0xFF00 {
                    self.pc = self.addr;
                    self.cycle = 0;
                }
            }
            4 => {
                self.pc = self.addr;
                self.cycle = 0;
            }
            _ => panic!()
        }
    }

    fn zeropage_indexed(&mut self, i: u8, access: AccessType) {
        match access {
            AccessType::Read => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.bus.read(self.addr); // QUIRK
                    self.addr = (self.addr as u8).wrapping_add(i) as u16;
                }
                4 => {
                    self.cycle = 0;
                    let value = self.bus.read(self.addr);
                    self.execute_read(value);
                }
                _ => panic!()
            }
            AccessType::ReadModifyWrite => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.bus.read(self.addr); // QUIRK
                    self.addr = (self.addr as u8).wrapping_add(i) as u16;
                }
                4 => {
                    self.tmp = self.bus.read(self.addr);
                }
                5 => {
                    self.bus.write(self.addr, self.tmp); // QUIRK
                    self.tmp = self.execute_rmw(self.tmp);
                }
                6 => {
                    self.cycle = 0;
                    self.bus.write(self.addr, self.tmp);
                }
                _ => panic!()
            },
            AccessType::Write => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                },
                3 => {
                    let _ = self.bus.read(self.addr); //TODO ?
                    self.addr = (self.addr as u8).wrapping_add(i) as u16;
                }
                4 => {
                    self.cycle = 0;
                    self.execute_write(self.addr);
                }
                _ => panic!()
            },
        }
    }

    fn implied(&mut self) {
        match self.cycle {
            2 => {
                self.bus.read(self.pc);
                self.cycle = 0;

                match self.instruction.name {
                    Mnemonic::Sei => self.sr |= sr_flags::INTERRUPT,
                    Mnemonic::Cli => self.sr &= !sr_flags::INTERRUPT,
                    Mnemonic::Sed => self.sr |= sr_flags::DECIMAL,
                    Mnemonic::Cld => self.sr &= !sr_flags::DECIMAL,
                    Mnemonic::Clc => self.sr &= !sr_flags::CARRY,
                    Mnemonic::Clv => self.sr &= !sr_flags::OVERFLOW,
                    Mnemonic::Tay => { self.y = self.a; self.update_flags(self.y); }
                    Mnemonic::Tax => { self.x = self.a; self.update_flags(self.x); }
                    Mnemonic::Txa => { self.a = self.x; self.update_flags(self.a); }
                    Mnemonic::Tya => { self.a = self.y; self.update_flags(self.a); }
                    Mnemonic::Txs => self.sp = self.x,
                    Mnemonic::Tsx => { self.x = self.sp; self.update_flags(self.x); }
                    Mnemonic::Inx => { self.x = self.x.wrapping_add(1); self.update_flags(self.x); }
                    Mnemonic::Iny => { self.y = self.y.wrapping_add(1); self.update_flags(self.y); }
                    Mnemonic::Dex => { self.x = self.x.wrapping_sub(1); self.update_flags(self.x); }
                    Mnemonic::Dey => { self.y = self.y.wrapping_sub(1); self.update_flags(self.y); }
                    Mnemonic::Nop => (),
                    Mnemonic::Sec => self.sr |= sr_flags::CARRY,
                    _ => panic!()
                }
            }
            _ => panic!()
        }
    }

    fn immediate(&mut self) {
        let value = self.bus.read(self.pc);
        self.pc += 1;
        self.execute_read(value);
        self.cycle = 0;
    }

    fn absolute(&mut self, access: AccessType) {
        match access {
            AccessType::Read => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.addr |= (self.bus.read(self.pc) as u16) << 8;
                    self.pc += 1;

                    if let Mnemonic::Jmp = self.instruction.name {
                        self.cycle = 0;
                        self.pc = self.addr;
                    }
                }
                4 => {
                    self.cycle = 0;
                    let value = self.bus.read(self.addr);
                    self.execute_read(value);
                }
                _ => panic!()
            },
            AccessType::ReadModifyWrite => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.addr |= (self.bus.read(self.pc) as u16) << 8;
                    self.pc += 1;
                }
                4 => { 
                    self.tmp = self.bus.read(self.addr);
                }
                5 => { 
                    self.bus.write(self.addr, self.tmp); // QUIRK
                    self.tmp = self.execute_rmw(self.tmp);
                }
                6 => {
                    self.cycle = 0;
                    self.bus.write(self.addr, self.tmp);
                }
                _ => panic!()
            },
            AccessType::Write => match self.cycle {
                2 => {
                    self.addr = self.bus.read(self.pc) as u16;
                    self.pc += 1;
                }
                3 => {
                    self.addr |= (self.bus.read(self.pc) as u16) << 8;
                    self.pc += 1;
                }
                4 => {
                    self.cycle = 0;
                    self.execute_write(self.addr);
                }
                _ => panic!()
            },
        }
    }

    fn set_addr_hi_index_lo(&mut self, hi: u8, i: u8) {
        self.addr |= (hi as u16) << 8;
        let lo;
        (lo, self.addr_invalid) = (self.addr as u8).overflowing_add(i);
        self.addr = (self.addr & 0xFF00) | lo as u16;
    }

    fn fix_addr(&mut self) {
        if self.addr_invalid {
            self.addr += 0x0100;
        }
    }

    fn set_ptr_hi_index_lo(&mut self, hi: u8, i: u8) {
        self.ptr |= (hi as u16) << 8;
        let lo;
        (lo, self.ptr_invalid) = (self.ptr as u8).overflowing_add(i);
        self.ptr = (self.ptr & 0xFF00) | lo as u16;
    }

    fn fix_ptr(&mut self) {
        if self.ptr_invalid {
            self.ptr += 0x0100;
        }
    }

    fn execute_read(&mut self, value: u8) {
        match self.instruction.name {
            Mnemonic::Lda => { self.a = value; self.update_flags(self.a); }
            Mnemonic::Ldx => { self.x = value; self.update_flags(self.x); }
            Mnemonic::Ldy => { self.y = value; self.update_flags(self.y); }
            Mnemonic::Ora => { self.a = self.a | value; self.update_flags(self.a); }
            Mnemonic::Eor => { self.a = self.a ^ value; self.update_flags(self.a); }
            Mnemonic::And => { self.a = self.a & value; self.update_flags(self.a); }
            Mnemonic::Cmp => self.compare(self.a, value),
            Mnemonic::Cpx => self.compare(self.x, value),
            Mnemonic::Cpy => self.compare(self.y, value),
            Mnemonic::Adc => self.adc(value),
            Mnemonic::Sbc => self.sbc(value),
            Mnemonic::Bit => {
                if value & self.a == 0 {
                    self.sr |= sr_flags::ZERO;
                } else {
                    self.sr &= !sr_flags::ZERO;
                }

                if value & 0b10000000 != 0 {
                    self.sr |= sr_flags::NEGATIVE;
                } else {
                    self.sr &= !sr_flags::NEGATIVE;
                }

                if value & 0b01000000 != 0 {
                    self.sr |= sr_flags::OVERFLOW;
                } else {
                    self.sr &= !sr_flags::OVERFLOW;
                }
            }
            _ => panic!()
        }
    }

    fn execute_write(&mut self, addr: u16) {
        match self.instruction.name {
            Mnemonic::Sta => self.bus.write(addr, self.a),
            Mnemonic::Stx => self.bus.write(addr, self.x),
            Mnemonic::Sty => self.bus.write(addr, self.y),
            _ => panic!()
        }
    }

    fn adc(&mut self, value: u8) {
        if self.sr & sr_flags::DECIMAL == 0 {
            self.a = self.add_bin(self.a, value, self.sr & sr_flags::CARRY != 0);
        } else {
            self.a = self.add_bcd(self.a, value, self.sr & sr_flags::CARRY != 0);
        }
    }

    fn sbc(&mut self, value: u8) {
        if self.sr & sr_flags::DECIMAL == 0 {
            self.a = self.add_bin(self.a, !value, self.sr & sr_flags::CARRY != 0);
        } else {
            self.a = self.sub_bcd(self.a, value, self.sr & sr_flags::CARRY != 0);
        }
    }

    fn compare(&mut self, reg: u8, operand: u8) {
        let diff = reg.wrapping_sub(operand);
        self.update_flags(diff);

        if reg >= operand {
            self.sr |= sr_flags::CARRY;
        } else {
            self.sr &= !sr_flags::CARRY;
        }
    }

    fn add_bin(&mut self, a: u8, b: u8, carry: bool) -> u8 {
        let c = a as u16 + b as u16 + carry as u16;

        if c & 0xFF00 != 0 {
            self.sr |= sr_flags::CARRY;
        } else {
            self.sr &= !sr_flags::CARRY;
        }

        if (a & 0x80) ^ (b & 0x80) == 0 && (c as u8 & 0x80) != (a & 0x80) {
            self.sr |= sr_flags::OVERFLOW;
        } else {
            self.sr &= !sr_flags::OVERFLOW;
        }

        self.update_flags(c as u8);
        c as u8
    }

    fn add_bcd(&mut self, a: u8, b: u8, carry: bool) -> u8 {
        let mut lo = (a & 0x0F) + (b & 0x0F) + carry as u8;
        let mut hi = (a & 0xF0) as u16 + (b & 0xF0) as u16;

        self.sr &= !(sr_flags::CARRY | sr_flags::OVERFLOW);

        if lo > 0x09 {
            hi += 0x10;
            lo += 0x06;
        }

        if !(a ^ b) & (a ^ hi as u8) & 0x80 != 0 {
            self.sr |= sr_flags::OVERFLOW;
        }

        if hi > 0x90 {
            hi += 96;
        }

		if hi >> 8 != 0 {
            self.sr |= sr_flags::CARRY;
        }

        let c = (lo & 0x0F) | (hi as u8 & 0xF0);
        self.update_flags(c);
        c
    }

    fn sub_bcd(&mut self, a: u8, b: u8, carry: bool) -> u8 {
        let tmp = (a as u16).wrapping_sub(b as u16).wrapping_sub(!carry as u16);

        let mut lo = (a & 0x0F).wrapping_sub(b & 0x0F).wrapping_sub(!carry as u8);
        let mut hi = (a & 0xF0) as u16 - (b & 0xF0) as u16;

        self.sr &= !(sr_flags::CARRY | sr_flags::OVERFLOW);

        if lo & 0x10 != 0 {
            lo = lo.wrapping_sub(6);
            hi = hi.wrapping_sub(1);
        }

        if hi & 0x0100 != 0 {
            hi -= 0x60;
        }

        if tmp >> 8 == 0 {
            self.sr |= sr_flags::CARRY;
        }

        if (a ^ b) & (a ^ tmp as u8) & 0x80 != 0 {
            self.sr |= sr_flags::OVERFLOW;
        }

        let c = (lo & 0x0F) | (hi as u8 & 0xF0);
        self.update_flags(c);
        c
    }
}