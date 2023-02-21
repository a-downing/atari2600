const PA7_FLAG: u8 = 1 << 6;
const TIMER_FLAG: u8 = 1 << 7;

enum PA7EdgeDetect {
    Positive,
    Negative
}

pub struct Riot {
    ram: [u8; 128],
    timer_cnt: u16,
    timer_value: u8,
    timer_interval: u16,
    timer_irq_enable: bool,
    interrupt_flag: u8,
    porta: u8,
    portb: u8
}

impl Riot {
    pub fn new() -> Self {
        Riot {
            ram: [0; 128],
            timer_cnt: 0,
            timer_value: 0,
            timer_interval: 1024,
            timer_irq_enable: false,
            interrupt_flag: 0,
            porta: !((1 << 3) | (1 << 7)),
            portb: 0
        }
    }

    pub fn switch_color(&mut self, enabled: bool) {
        if enabled {
            self.portb |= 1 << 3;
        } else {
            self.portb &= !(1 << 3);
        }
    }

    pub fn switch_select(&mut self, enabled: bool) {
        if enabled {
            self.portb |= 1 << 1;
        } else {
            self.portb &= !(1 << 1);
        }
    }

    pub fn switch_reset(&mut self, enabled: bool) {
        if enabled {
            self.portb |= 1 << 0;
        } else {
            self.portb &= !(1 << 0);
        }
    }

    pub fn irq(&self) -> bool {
        self.timer_irq_enable && self.interrupt_flag & TIMER_FLAG != 0
    }

    pub fn cycle(&mut self) {
        if self.timer_value == 0x00 {
            self.interrupt_flag |= TIMER_FLAG;
        }

        if self.timer_cnt == self.timer_interval {
            if self.timer_value == 0xFF {
                self.timer_interval = 1;
                self.timer_cnt = 0;
            }

            self.timer_value = self.timer_value.wrapping_sub(1);
            self.timer_cnt = 0;
        }

        self.timer_cnt += 1;
    }

    fn read_ram(&self, addr: u16) -> u8 {
        self.ram[addr as usize & 0x7F]
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr & (1 << 9) {
            0 => self.read_ram(addr),
            _ => match addr & (1 << 2) {
                0 => match addr & 0x1287 {
                    0x0283 => todo!(), //data-direction control for I/O register B
                    0x0282 => self.portb, //I/O register B
                    0x0281 => todo!(), //data-direction control for I/O register A
                    0x0280 => self.porta, //I/O register A
                    _ => panic!("Unknown RIOT read register: 0x{:04X}", addr)
                }
                _ => match addr & (1 << 0) {
                    0 => match addr & 0x128d {
                        0x028C => { //enable the timer interrupt and read the timer
                            self.timer_irq_enable = true;
                            self.interrupt_flag &= !TIMER_FLAG;
                            self.timer_value
                        }
                        0x0284 => { //disable the timer interrupt and read the timer
                            self.timer_irq_enable = false;
                            self.interrupt_flag &= !TIMER_FLAG;
                            self.timer_value
                        }
                        _ => panic!("Unknown RIOT read register: 0x{:04X}", addr)
                    }
                    _ => match addr & 0x1285 {
                        0x0285 => { //read the interrupt flags
                            let value = self.interrupt_flag;
                            self.interrupt_flag &= !PA7_FLAG;
                            value
                        }
                        _ => panic!("Unknown RIOT read register: 0x{:04X}", addr)
                    }
                }
            }
        }
    }

    fn write_ram(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize & 0x7F] = value
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr & (1 << 9) {
            0 => self.write_ram(addr, value),
            _ => match addr & (1 << 2) {
                0 => match addr & 0x1287 {
                    0x0283 => todo!(), //data-direction control for I/O register B
                    0x0282 => todo!(), //I/O register B
                    0x0281 => todo!(), //data-direction control for I/O register A
                    0x0280 => todo!(), //I/O register A
                    _ => panic!("Unknown RIOT write register: 0x{:04X}", addr)
                }
                _ => match addr & (1 << 4) {
                    0 => match addr & 0x1297 {
                        //0x0287 => self.pa7_cfg(true, PA7EdgeDetect::Positive), //enable the PA7 interrupt and select positive edge-detect
                        //0x0286 => self.pa7_cfg(true, PA7EdgeDetect::Negative), //enable the PA7 interrupt and select negative edge-detect
                        0x0285 => self.pa7_cfg(false, PA7EdgeDetect::Positive), //disable the PA7 interrupt and select positive edge-detect
                        0x0284 => self.pa7_cfg(false, PA7EdgeDetect::Negative), //disable the PA7 interrupt and select negative edge-detect
                        _ => panic!("Unknown RIOT write register: 0x{:04X}", addr)
                    }
                    _ => match addr & 0x129f {
                        //0x029F => self.timer_cfg(true, value, 1024), //enable the timer interrupt and set the timer using the 1024-cycle interval
                        //0x029E => self.timer_cfg(true, value, 64), //enable the timer interrupt and set the timer using the 64-cycle interval
                        //0x029D => self.timer_cfg(true, value, 8), //enable the timer interrupt and set the timer using the 8-cycle interval
                        //0x029C => self.timer_cfg(true, value, 1), //enable the timer interrupt and set the timer using the 1-cycle interval
                        0x0297 => self.timer_cfg(false, value, 1024), //disable the timer interrupt and set the timer using the 1024-cycle interval
                        0x0296 => self.timer_cfg(false, value, 64), //disable the timer interrupt and set the timer using the 64-cycle interval
                        0x0295 => self.timer_cfg(false, value, 8), //disable the timer interrupt and set the timer using the 8-cycle interval
                        0x0294 => self.timer_cfg(false, value, 1), //disable the timer interrupt and set the timer using the 1-cycle interval
                        _ => panic!("Unknown RIOT write register: 0x{:04X}", addr)
                    }
                }
            }
        }
    }

    fn pa7_cfg(&mut self, enable: bool, detect: PA7EdgeDetect) {
        unimplemented!();
    }

    fn timer_cfg(&mut self, enable: bool, value: u8, interval: u16) {
        self.timer_irq_enable = enable;
        self.timer_value = value;
        self.timer_interval = interval;
        self.interrupt_flag &= !TIMER_FLAG;
    }
}