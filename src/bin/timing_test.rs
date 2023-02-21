use atari2600::{ AddressBus, MOS6502 };

struct SystemBus {
    ram: Vec<u8>
}

impl AddressBus for SystemBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0xFFFC => 0x00,
            0xFFFD => 0x10,
            _ => self.ram[addr as usize]
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut rom = std::fs::read(&args[1]).unwrap();

    rom.resize(1024 * 8, 0);

    let system = SystemBus { ram: rom };

    let mut cpu = MOS6502::new(system);
    cpu.reset();

    loop {
        let cycles = cpu.cycle(true);
    }
}