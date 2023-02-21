use crate::{riot, tia, AddressBus};

pub struct Atari2600 {
    pub rom: Vec<u8>,
    pub riot: riot::Riot,
    pub tia: tia::Tia
}

#[derive(Debug)]
enum Atari2600Chip {
    Cartridge,
    RIOT,
    TIA
}

mod addresses {
    pub const CART_MASK: u16 = 1 << 12;
    pub const CART_SELECT: u16 = 1 << 12;
    pub const RIOT_MASK: u16 = 1 << 12 | 1 << 7;
    pub const RIOT_SELECT: u16 = 1 << 7;
    pub const TIA_MASK: u16 = 1 << 12 | 1 << 7;
    pub const TIA_SELECT: u16 = 0;
}

impl Atari2600 {
    pub fn new(rom: Vec<u8>) -> Self {
        Atari2600 { rom, riot: riot::Riot::new(), tia: tia::Tia::new() }
    }

    fn decode(addr: u16) -> Atari2600Chip {
        if addr & addresses::CART_MASK == addresses::CART_SELECT {
            Atari2600Chip::Cartridge
        } else if addr & addresses::RIOT_MASK == addresses::RIOT_SELECT {
            Atari2600Chip::RIOT
        } else if addr & addresses::TIA_MASK == addresses::TIA_SELECT {
            Atari2600Chip::TIA
        } else {
            unreachable!();
        }
    }
}

impl AddressBus for Atari2600 {
    fn read(&mut self, addr: u16) -> u8 {
        let chip = Atari2600::decode(addr);

        match chip {
            Atari2600Chip::Cartridge => self.rom[addr as usize & 0xFFF],
            Atari2600Chip::RIOT => self.riot.read(addr),
            Atari2600Chip::TIA => self.tia.read(addr)
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        let chip = Atari2600::decode(addr);

        match chip {
            Atari2600Chip::Cartridge => unimplemented!(),
            Atari2600Chip::RIOT => self.riot.write(addr, value),
            Atari2600Chip::TIA => self.tia.write(addr, value)
        }
    }
}