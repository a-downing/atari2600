pub const NUM_SCANLINES: u16 = 262;
pub const CLOCKS_PER_SCANLINE: u16 = 228;

pub struct Tia {
    pub frame: [u8; (CLOCKS_PER_SCANLINE * NUM_SCANLINES) as usize],
    draw: bool,
    scanline: u16,
    clock: u16,
    vblank: u8,
    wsync: bool,
    vdelp1: u8,
    vdelp0: u8,
    hmp1: u8,
    hmp0: u8,
    resp0: u16,
    resp1: u16,
    grp0: u8,
    grp0a: u8,
    grp1: u8,
    grp1a: u8,
    pf0: u8,
    pf1: u8,
    pf2: u8,
    refp1: u8,
    refp0: u8,
    ctrlpf: u8,
    colubk: u8,
    colupf: u8,
    colup1: u8,
    colup0: u8,
    nusiz1: u8,
    nusiz0: u8,
    inpt5: u8,
    inpt4: u8
}

impl Tia {
    pub fn new() -> Self {
        Tia {
            frame: [0; (CLOCKS_PER_SCANLINE * NUM_SCANLINES) as usize],
            draw: false,
            scanline: 0,
            clock: 0,
            vblank: 0,
            wsync: false,
            vdelp1: 0,
            vdelp0: 0,
            hmp1: 0,
            hmp0: 0,
            resp0: 68,
            resp1: 68,
            grp0: 0,
            grp0a: 0,
            grp1: 0,
            grp1a: 0,
            pf0: 0,
            pf1: 0,
            pf2: 0,
            refp1: 0,
            refp0: 0,
            ctrlpf: 0,
            colubk: 0,
            colupf: 0,
            colup1: 0,
            colup0: 0,
            nusiz1: 0,
            nusiz0: 0,
            inpt5: 0,
            inpt4: 0
        }
    }

    pub fn wsync(&self) -> bool {
        self.wsync
    }

    pub fn draw(&self) -> bool {
        self.draw
    }

    pub fn input4(&mut self, value: u8) {
        self.inpt4 = value;
    }

    pub fn input5(&mut self, value: u8) {
        self.inpt5 = value;
    }

    pub fn drew(&mut self) {
        self.draw = false;
    }

    fn playfield_pixel(&self, index: u16, reflect: bool) -> bool {
        if reflect {
            match index {
                0..=7 => self.pf2 & (1 << (7 - index)) != 0,
                8..=15 => self.pf1 & (1 << (index - 8)) != 0,
                16..=19 => self.pf0 & (1 << (7 - (index - 16))) != 0,
                _ => panic!()
            }
        } else {
            match index {
                0..=3 => self.pf0 & (1 << (index + 4)) != 0,
                4..=11 => self.pf1 & (1 <<  (7 - (index - 4))) != 0,
                12..=19 => self.pf2 & (1 << (index - 12)) != 0,
                _ => panic!()
            }
        }
    }

    fn player_pixel(&self, clock: u16, player: u8, pos: u16, size: u16, reflect: bool) -> bool {
        if clock >= pos && clock - pos < 8 * size {
            let index = (clock - pos) / size;

            return if !reflect {
                player & (1 << (7 - index)) != 0
            } else {
                player & (1 << index) != 0
            };
        }

        false
    }

    fn player_pixel_extra(&self, clock: u16, player: u8, pos: u16, reflect: bool) -> bool {
        match self.nusiz0 & 0b111 {
            0 => self.player_pixel(clock, player, pos, 1, reflect),
            1 => {
                self.player_pixel(clock, player, pos, 1, reflect)
                || self.player_pixel(clock, player, pos + 16, 1, reflect)
            }
            2 => {
                self.player_pixel(clock, player, pos, 1, reflect)
                || self.player_pixel(clock, player, pos + 32, 1, reflect)
            }
            3 => {
                self.player_pixel(clock, player, pos, 1, reflect)
                || self.player_pixel(clock, player, pos + 16, 1, reflect)
                || self.player_pixel(clock, player, pos + 32, 1, reflect)
            }
            4 => {
                self.player_pixel(clock, player, pos, 1, reflect)
                || self.player_pixel(clock, player, pos + 64, 1, reflect)
            }
            5 => self.player_pixel(clock, player, pos, 2, reflect),
            6 => {
                self.player_pixel(clock, player, pos, 1, reflect)
                || self.player_pixel(clock, player, pos + 32, 1, reflect)
                || self.player_pixel(clock, player, pos + 64, 1, reflect)
            }
            7 => self.player_pixel(clock, player, pos, 4, reflect),
            _ => false
        }
    }

    fn player_graphic(&self, player0: bool) -> u8 {
        if player0 {
            if self.vdelp0 != 0 {
                self.grp0a
            } else {
                self.grp0
            }
        } else {
            if self.vdelp1 != 0{
                self.grp1a
            } else {
                self.grp1
            }
        }
    }

    pub fn cycle(&mut self) {
        if self.clock == CLOCKS_PER_SCANLINE {
            self.clock = 0;
            self.scanline += 1;
            self.wsync = false;
        }

        let index = self.scanline as usize * CLOCKS_PER_SCANLINE as usize + self.clock as usize;

        if index >= self.frame.len() {
            self.clock += 1;
            return;
        }

        if self.vblank & (1 << 1) != 0 || self.clock < 68 {
            self.frame[index as usize] = 0;
            self.clock += 1;
            return;
        }

        let x = self.clock - 68;
        let pf_index = x / 4;

        let pixel = if pf_index < 20 {
            self.playfield_pixel(pf_index, false)
        } else {
            let reflect = self.ctrlpf & 0x01 != 0;
            self.playfield_pixel(pf_index - 20, reflect)
        };

        self.frame[index as usize] = if pixel { self.colupf } else { self.colubk };

        if self.player_pixel_extra(self.clock, self.player_graphic(true), self.resp0, self.refp0 & (1 << 3) != 0) {
            self.frame[index as usize] = self.colup0;
        }

        if self.player_pixel_extra(self.clock, self.player_graphic(false), self.resp1, self.refp1 & (1 << 3) != 0) {
            self.frame[index as usize] = self.colup1;
        }

        self.clock += 1;

        if self.clock == CLOCKS_PER_SCANLINE {
            self.wsync = false;
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr & 0x108f {
            0x000D => self.inpt5, //INPT5 (input port 5, trigger 1)
            0x000C => self.inpt4, //INPT4 (input port 4, trigger 0)
            0x000B => 0, //INPT3 (input port 3, pot 3)
            0x000A => 0, //INPT2 (input port 2, pot 2)
            0x0009 => 0, //INPT1 (input port 1, pot 1)
            0x0008 => 0, //INPT0 (input port 0, pot 0)
            0x0007 => 0, //CXPPMM (collision of players and missiles)
            0x0006 => 0, //CXBLPF (collision of ball with playfield)
            0x0005 => 0, //CXM1FB (collision of missile 1 with playfield-ball)
            0x0004 => 0, //CXM0FB (collision of missile 0 with playfield-ball)
            0x0003 => 0, //CXP1FB (collision of player 1 with playfield-ball)
            0x0002 => 0, //CXP0FB (collision of player 0 with playfield-ball)
            0x0001 => 0, //CXM1P (collision of missile 1 with players)
            0x0000 => 0, //CXM0P (collision of missile 0 with players)
            _ => panic!("Unknown TIA read register: 0x{:04X}", addr)
        }
    }

    fn hmove(&self, resp: u16, hm: u8) -> u16 {
        let hm = hm >> 4;

        if hm & (1 << 3) == 0 {
            resp.wrapping_sub(hm as u16)
        } else {
            resp.wrapping_add(!(0xFFF0 | hm as u16) + 1)
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr & 0x10bf {
            0x002D..=0x003F => (), //???
            0x002C => (), //CXCLR (clear collision latches)
            0x002B => {   //HMCLR (clear horizontal motion registers)
                self.hmp0 = 0;
                self.hmp1 = 0;
            }
            0x002A => {   //HMOVE (apply horizontal motion)
                self.resp0 = self.hmove(self.resp0, self.hmp0);
                self.resp1 = self.hmove(self.resp1, self.hmp1);
            }
            0x0029 => (), //RESMP1 (reset missile 1 to player 1)
            0x0028 => (), //RESMP0 (reset missile 0 to player 0)
            0x0027 => (), //VDELBL (vertical delay ball)
            0x0026 => self.vdelp1 = value, //VDELP1 (vertical delay player 1)
            0x0025 => self.vdelp0 = value, //VDELP0 (vertical delay player 0)
            0x0024 => (), //HMBL (horizontal motion ball)
            0x0023 => (), //HMM1 (horizontal motion missile 1)
            0x0022 => (), //HMM0 (horizontal motion missile 0)
            0x0021 => self.hmp1 = value, //HMP1 (horizontal motion player 1)
            0x0020 => self.hmp0 = value, //HMP0 (horizontal motion player 0)
            0x001F => (), //ENABL (enable ball graphics)
            0x001E => (), //ENAM1 (enable missile 1 graphics)
            0x001D => (), //ENAM0 (enable missile 0 graphics)
            0x001C => {   //GRP1 (graphics player 1)
                self.grp1 = value;
                self.grp0a = self.grp0;
            }
            0x001B => {   //GRP0 (graphics player 0)
                self.grp0 = value;
                self.grp1a = self.grp1;
                }
            0x001A => (), //AUDV1 (audio volume 1)
            0x0019 => (), //AUDV0 (audio volume 0)
            0x0018 => (), //AUDF1 (audio frequency 1)
            0x0017 => (), //AUDF0 (audio frequency 0)
            0x0016 => (), //AUDC1 (audio control 1)
            0x0015 => (), //AUDC0 (audio control 0)
            0x0014 => (), //RESBL (reset ball)
            0x0013 => (), //RESM1 (reset missile 1)
            0x0012 => (), //RESM0 (reset missile 0)
            0x0011 => self.resp1 = self.clock.max(68) + 5, //RESP1 (reset player 1)
            0x0010 => self.resp0 = self.clock.max(68) + 5, //RESP0 (reset player 0)
            0x000F => self.pf2 = value, //PF2 (playfield register byte 2)
            0x000E => self.pf1 = value, //PF1 (playfield register byte 1)
            0x000D => self.pf0 = value, //PF0 (playfield register byte 0)
            0x000C => self.refp1 = value, //REFP1 (reflect player 1)
            0x000B => self.refp0 = value, //REFP0 (reflect player 0)
            0x000A => self.ctrlpf = value, //CTRLPF (control playfield ball size and reflect)
            0x0009 => self.colubk = value, //COLUBK (color-lum background)
            0x0008 => self.colupf = value, //COLUPF (color-lum playfield)
            0x0007 => self.colup1 = value, //COLUP1 (color-lum player 1)
            0x0006 => self.colup0 = value, //COLUP0 (color-lum player 0)
            0x0005 => self.nusiz1 = value, //NUSIZ1 (number-size player-missile 1)
            0x0004 => self.nusiz0 = value, //NUSIZ0 (number-size player-missile 0)
            0x0003 => self.clock = 0, //RSYNC (reset horizontal sync counter)
            0x0002 => self.wsync = true, //WSYNC (wait for leading edge of horizontal blank)
            0x0001 => self.vblank = value, //VBLANK (vertical blank set-clear)
            0x0000 => { //VSYNC (vertical sync set-clear)
                if value & (1 << 1) != 0 {
                    self.scanline = 0;
                    self.draw = true;
                }
            }
            _ => panic!("Unknown TIA write register: 0x{:04X}", addr)
        }
    }
}