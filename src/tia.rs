use std::collections::VecDeque;

pub const NUM_SCANLINES: u16 = 262;
pub const CLOCKS_PER_SCANLINE: u16 = 228;

#[derive(Clone, Copy, Debug)]
pub struct TiaAudioSample {
    pub value: u8,
    pub cycles: u16
}

pub struct Tia {
    pub frame: [u8; (CLOCKS_PER_SCANLINE * NUM_SCANLINES) as usize],
    pub audio: [VecDeque<TiaAudioSample>; 2],
    draw: bool,
    scanline: u16,
    ctr: u16,
    color_clock: u16,
    audio_div_ctr: [u8; 2],
    audio_div3_ctr: u8,
    lfsr4: [u8; 2],
    lfsr5: [u8; 2],
    lfsr9: [u16; 2],
    vblank: u8,
    wsync: bool,
    resmp1: u8,
    vdelbl: u8,
    vdelp1: u8,
    vdelp0: u8,
    hmbl: u8,
    hmm1: u8,
    hmm0: u8,
    hmp1: u8,
    hmp0: u8,
    enabl: u8,
    enabla: u8,
    enam1: u8,
    enam0: u8,
    resbl: u16,
    resm1: u16,
    resm0: u16,
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
    inpt4: u8,
    audv: [u8; 2],
    audf: [u8; 2],
    audc: [u8; 2],
    cxppmm: u8,
    cxblpf: u8,
    cxm1fb: u8,
    cxm0fb: u8,
    cxp1fb: u8,
    cxp0fb: u8,
    cxm1p: u8,
    cxm0p: u8,
    resmp0: u8,
    p0_cnt: Counter,
    p0_pixel: GraphicsCounter,
    p1_cnt: Counter,
    p1_pixel: GraphicsCounter,
}

impl Tia {
    pub fn new() -> Self {
        Tia {
            frame: [0; (CLOCKS_PER_SCANLINE * NUM_SCANLINES) as usize],
            audio: [VecDeque::new(), VecDeque::new()],
            draw: false,
            scanline: 0,
            ctr: 0,
            color_clock: 0,
            audio_div_ctr: [0; 2],
            audio_div3_ctr: 0,
            lfsr4: [0xFF; 2],
            lfsr5: [0xFF; 2],
            lfsr9: [0xFFFF; 2],
            vblank: 0,
            wsync: false,
            resmp1: 0,
            vdelbl: 0,
            vdelp1: 0,
            vdelp0: 0,
            hmbl: 0,
            hmm1: 0,
            hmm0: 0,
            hmp1: 0,
            hmp0: 0,
            enabl: 0,
            enabla: 0,
            enam1: 0,
            enam0: 0,
            resbl: 68,
            resm1: 68,
            resm0: 68,
            p0_cnt: Counter::new(0),
            p1_cnt: Counter::new(0),
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
            inpt4: 0x80,
            audv: [0; 2],
            audf: [0; 2],
            audc: [0; 2],
            cxppmm: 0,
            cxblpf: 0,
            cxm1fb: 0,
            cxm0fb: 0,
            cxp1fb: 0,
            cxp0fb: 0,
            cxm1p: 0,
            cxm0p: 0,
            resmp0: 0,
            p0_pixel: GraphicsCounter::new(),
            p1_pixel: GraphicsCounter::new(),
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

    fn player_pixel_extra(&self, clock: u16, player: u8, pos: u16, reflect: bool, nusiz: u8) -> bool {
        match nusiz & 0b111 {
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

    fn audio_sample(&self, chan: usize, polarity: bool) -> u8 {
        if polarity { 128 + (self.audv[chan] << 3) } else { 128 - (self.audv[chan] << 3) }
    }

    pub fn audio_cycle(&mut self, chan: usize) -> u8 {
        // The actual LFSR is lfsr[5:1], lfsr[0] is the previous output
        let lfsr5_prev_out = self.lfsr5[chan] & 1 == 1;
        self.lfsr5[chan] = (((self.lfsr5[chan] >> 1) | ((self.lfsr5[chan] >> 2) ^ (self.lfsr5[chan] >> 0) & 1) << 5)) & 0b111111;
        let lfsr5_out = self.lfsr5[chan] & 1 == 1;
        
        // no need for previous output
        let lfsr9_out = self.lfsr9[chan] & 1 == 1;
        self.lfsr9[chan] = (((self.lfsr9[chan] >> 1) | ((self.lfsr9[chan] >> 4) ^ (self.lfsr9[chan] >> 0) & 1) << 8)) & 0b111111111;

        let modified_clock = match self.audc[chan] {
            0x2 => self.lfsr5[chan] >> 1 == 1 || self.lfsr5[chan] >> 1 == 15, // happens every ~15 cycles
            0x3 | 0x7 | 0xF => lfsr5_out && !lfsr5_prev_out, // rising edge
            0x6 | 0xA | 0xE => self.lfsr5[chan] >> 1 == 1, // happens every 31 cycles
            _ => true
        };

        let lfsr4_out = self.lfsr4[chan] & 1 == 1;

        if modified_clock {
            match self.audc[chan] {
                0x4 | 0x5 | 0x7 | 0xC | 0xD | 0xF => self.lfsr4[chan] = (((self.lfsr4[chan] >> 1) | ((self.lfsr4[chan] >> 3) ^ 1 & 1) << 3)) & 0b1111,
                _ => self.lfsr4[chan] = (((self.lfsr4[chan] >> 1) | ((self.lfsr4[chan] >> 1) ^ (self.lfsr4[chan] >> 0) & 1) << 3)) & 0b1111
            }
        }

        match self.audc[chan] {
            0x0 => {
                self.lfsr4[chan] = 0b1111;
                self.lfsr5[chan] = 0b11111;
                self.lfsr9[chan] = 0b111111111;
                128
            },
            0x1 => self.audio_sample(chan, lfsr4_out),
            0x2 => self.audio_sample(chan, lfsr4_out), // /15 4-bit wtf?
            0x3 => self.audio_sample(chan, lfsr4_out),
            0x4 => self.audio_sample(chan, lfsr4_out),
            0x5 => self.audio_sample(chan, lfsr4_out),
            0x6 => self.audio_sample(chan, lfsr5_out),
            0x7 => self.audio_sample(chan, lfsr4_out),
            0x8 => self.audio_sample(chan, lfsr9_out),
            0x9 => self.audio_sample(chan, lfsr5_out),
            0xA => self.audio_sample(chan, lfsr5_out),
            0xB => {
                self.lfsr4[chan] = 0b1111;
                self.lfsr9[chan] = 0b000001111;
                128
            }
            0xC => self.audio_sample(chan, lfsr4_out),
            0xD => self.audio_sample(chan, lfsr4_out),
            0xE => self.audio_sample(chan, lfsr5_out),
            0xF => self.audio_sample(chan, lfsr4_out),
            _ => unreachable!()
        }
    }

    pub fn audio_clockgen(&mut self) {
        for chan in 0..=1 {
            if self.audc[chan] & 0b1100 == 0b1100 {
                if self.audio_div3_ctr == 0 {
                    if self.audio_div_ctr[chan] >= self.audf[chan] {
                        let value = self.audio_cycle(chan);
                        self.audio[chan].push_back(TiaAudioSample { value, cycles: self.ctr });
                        self.audio_div_ctr[chan] = 0xFF;
                    }

                    self.audio_div_ctr[chan] = self.audio_div_ctr[chan].wrapping_add(1);
                }
            } else {
                if self.audio_div_ctr[chan] >= self.audf[chan] {
                    let value = self.audio_cycle(chan);
                    self.audio[chan].push_back(TiaAudioSample { value, cycles: self.ctr });
                    self.audio_div_ctr[chan] = 0xFF;
                }

                self.audio_div_ctr[chan] = self.audio_div_ctr[chan].wrapping_add(1);
            }
        }

        self.audio_div3_ctr += 1;

        if self.audio_div3_ctr == 3 {
            self.audio_div3_ctr = 0;
        }
    }

    fn player_pixel2(&self, grp: u8, pixel: u8, refp: u8) -> bool {
        if refp & (1 << 3) == 0 {
            grp & (1 << pixel) != 0
        } else {
            grp & (1 << (7 - pixel)) != 0
        }
    }

    fn player_pixel_clock_div(nusiz: u8) -> u8 {
        match nusiz & 0b111 {
            0b111 => 4,
            0b101 => 2,
            _ => 1
        }
    }

    pub fn cycle(&mut self) {
        self.ctr = self.ctr.wrapping_add(1);

        if self.color_clock == CLOCKS_PER_SCANLINE {
            self.color_clock = 0;
            self.scanline += 1;
            self.wsync = false;
        }

        if self.color_clock == 0 || self.color_clock == 114 {
            self.audio_clockgen();
        }

        let index = self.scanline as usize * CLOCKS_PER_SCANLINE as usize + self.color_clock as usize;

        if index >= self.frame.len() {
            self.color_clock += 1;
            return;
        }

        if self.vblank & (1 << 1) != 0 || self.color_clock < 68 {
            self.frame[index as usize] = 0;
            self.color_clock += 1;
            return;
        }

        let x = self.color_clock - 68;
        let pf_index = x / 4;

        let pf_pixel = if pf_index < 20 {
            self.playfield_pixel(pf_index, false)
        } else {
            let reflect = self.ctrlpf & 0x01 != 0;
            self.playfield_pixel(pf_index - 20, reflect)
        };

        self.p0_cnt.cycle();

        if self.p0_cnt.cmp_delayed(160, 1) {
            self.p0_pixel.reset();
        } else if self.p0_cnt.cmp_delayed(16, 1) && self.nusiz0 & 0b101 == 1 {
            self.p0_pixel.reset();
        } else if self.p0_cnt.cmp_delayed(32, 1) && self.nusiz0 & 0b110 == 2 {
            self.p0_pixel.reset();
        } else if self.p0_cnt.cmp_delayed(64, 1) && self.nusiz0 & 0b101 == 4 {
            self.p0_pixel.reset();
        }

        if self.p0_cnt.cmp(160) {
            self.p0_cnt.set(0);
        }

        let p0_pixel_bit = self.p0_pixel.cycle(Self::player_pixel_clock_div(self.nusiz0));

        let p0_pixel = if p0_pixel_bit != 0 {
            let grp = if self.vdelp0 != 0 {
                self.grp0a
            } else {
                self.grp0
            };

            self.player_pixel2(grp, p0_pixel_bit - 1, self.refp0)
        } else {
            false
        };

        self.p1_cnt.cycle();

        if self.p1_cnt.cmp_delayed(160, 1) {
            self.p1_pixel.reset();
        } else if self.p1_cnt.cmp_delayed(16, 1) && self.nusiz1 & 0b101 == 1 {
            self.p1_pixel.reset()
        } else if self.p1_cnt.cmp_delayed(32, 1) && self.nusiz1 & 0b110 == 2 {
            self.p1_pixel.reset()
        } else if self.p1_cnt.cmp_delayed(64, 1) && self.nusiz1 & 0b101 == 4 {
            self.p1_pixel.reset()
        }

        if self.p1_cnt.cmp(160) {
            self.p1_cnt.set(0);
        }

        let p1_pixel_bit = self.p1_pixel.cycle(Self::player_pixel_clock_div(self.nusiz1));

        let p1_pixel = if p1_pixel_bit != 0 {
            let grp = if self.vdelp1 != 0 {
                self.grp1a
            } else {
                self.grp1
            };

            self.player_pixel2(grp, p1_pixel_bit - 1, self.refp1)
        } else {
            false
        };

        let mut color: Option<u8> = None;
        let ball_size = (((self.ctrlpf >> 4) & 0b11) + 1) * 2;

        if self.ctrlpf & (1 << 2) != 0 && pf_pixel {
            color = Some(self.colupf);
        } else {
            if p0_pixel || p1_pixel {
                if p1_pixel {
                    color = Some(self.colup1);
                }

                if p0_pixel {
                    color = Some(self.colup0);
                }
            } else  if pf_pixel {
                color = Some(self.colupf);
            }
        }

        let bl_enable = if self.vdelbl == 0 { self.enabl != 0 } else { self.enabla != 0 };
        let bl_pixel = self.color_clock >= self.resbl && self.color_clock < self.resbl + ball_size as u16 && bl_enable;
        let m0_pixel = self.color_clock == self.resm0 && self.enam0 != 0 && self.resmp0 == 0;
        let m1_pixel = self.color_clock == self.resm1 && self.enam1 != 0 && self.resmp1 == 0;

        if color.is_none() && bl_pixel {
            color = Some(self.colupf);
        }

        if color.is_none() && m0_pixel {
            color = Some(self.colup0);
        }

        if color.is_none() && m1_pixel {
            color = Some(self.colup1);
        }

        self.frame[index as usize] = color.unwrap_or(self.colubk);

        if p0_pixel && p1_pixel {
            self.cxppmm |= 1 << 7;
        }

        if m0_pixel && m1_pixel {
            self.cxppmm |= 1 << 6;
        }

        if bl_pixel && pf_pixel {
            self.cxblpf |= 1 << 7;
        }

        if m1_pixel && pf_pixel {
            self.cxm1fb |= 1 << 7;
        }

        if m1_pixel && bl_pixel {
            self.cxm1fb |= 1 << 6;
        }

        if m0_pixel && pf_pixel {
            self.cxm0fb |= 1 << 7;
        }

        if m0_pixel && bl_pixel {
            self.cxm0fb |= 1 << 6;
        }

        if p1_pixel && pf_pixel {
            self.cxp1fb |= 1 << 7;
        }

        if p1_pixel && bl_pixel {
            self.cxp1fb |= 1 << 6;
        }

        if p0_pixel && pf_pixel {
            self.cxp0fb |= 1 << 7;
        }

        if p0_pixel && bl_pixel {
            self.cxp0fb |= 1 << 6;
        }

        if m1_pixel && p0_pixel {
            self.cxm1p |= 1 << 7;
        }

        if m1_pixel && p1_pixel {
            self.cxm1p |= 1 << 6;
        }

        if m0_pixel && p1_pixel {
            self.cxm0p |= 1 << 7;
        }

        if m0_pixel && p0_pixel {
            self.cxm0p |= 1 << 6;
        }

        self.color_clock += 1;

        if self.color_clock == CLOCKS_PER_SCANLINE {
            self.wsync = false;
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr & 0x108f {
            0x000F => 0, // asteroids writes here ??
            0x000E => 0, // asteroids writes here ??
            0x000D => self.inpt5, //INPT5 (input port 5, trigger 1)
            0x000C => self.inpt4, //INPT4 (input port 4, trigger 0)
            0x000B => 0, //INPT3 (input port 3, pot 3)
            0x000A => 0, //INPT2 (input port 2, pot 2)
            0x0009 => 0, //INPT1 (input port 1, pot 1)
            0x0008 => 0, //INPT0 (input port 0, pot 0)
            0x0007 => self.cxppmm, //CXPPMM (collision of players and missiles)
            0x0006 => self.cxblpf, //CXBLPF (collision of ball with playfield)
            0x0005 => self.cxm1fb, //CXM1FB (collision of missile 1 with playfield-ball)
            0x0004 => self.cxm0fb, //CXM0FB (collision of missile 0 with playfield-ball)
            0x0003 => self.cxp1fb, //CXP1FB (collision of player 1 with playfield-ball)
            0x0002 => self.cxp0fb, //CXP0FB (collision of player 0 with playfield-ball)
            0x0001 => self.cxm1p, //CXM1P (collision of missile 1 with players)
            0x0000 => self.cxm0p, //CXM0P (collision of missile 0 with players)
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

    fn hmove2(&self, resp: u16, hm: u8) -> u16 {
        let hm = hm >> 4;

        if hm & (1 << 3) == 0 {
            //resp.wrapping_add(hm as u16)
            modular_add(resp, hm as u16, 160)
        } else {
            //resp.wrapping_sub(!(0xFFF0 | hm as u16) + 1)
            modular_sub(resp, !(0xFFF0 | hm as u16) + 1, 160)
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr & 0x10bf {
            0x002D..=0x003F => (), //???
            0x002C => { //CXCLR (clear collision latches)
                self.cxppmm = 0;
                self.cxblpf = 0;
                self.cxm1fb = 0;
                self.cxm0fb = 0;
                self.cxp1fb = 0;
                self.cxp0fb = 0;
                self.cxm1p = 0;
                self.cxm0p = 0;
            }
            0x002B => {   //HMCLR (clear horizontal motion registers)
                self.hmbl = 0;
                self.hmm0 = 0;
                self.hmm1 = 0;
                self.hmp0 = 0;
                self.hmp1 = 0;
            }
            0x002A => {   //HMOVE (apply horizontal motion)
                self.resbl = self.hmove(self.resbl, self.hmbl);
                self.resm0 = self.hmove(self.resm0, self.hmm0);
                self.resm1 = self.hmove(self.resm1, self.hmm1);
                self.p0_cnt.set(self.hmove2(self.p0_cnt.value(), self.hmp0));
                self.p1_cnt.set(self.hmove2(self.p1_cnt.value(), self.hmp1));
            }
            0x0029 => { //RESMP1 (reset missile 1 to player 1)
                self.resmp1 = value & 2;
                self.resm1 = 0; // TODO fix
             }
            0x0028 => { //RESMP0 (reset missile 0 to player 0)
                self.resmp0 = value & 2;
                self.resm0 = 0 // TODO fix;
             }
            0x0027 => self.vdelbl = value & 1, //VDELBL (vertical delay ball)
            0x0026 => self.vdelp1 = value & 1, //VDELP1 (vertical delay player 1)
            0x0025 => self.vdelp0 = value & 1, //VDELP0 (vertical delay player 0)
            0x0024 => self.hmbl = value, //HMBL (horizontal motion ball)
            0x0023 => self.hmm1 = value, //HMM1 (horizontal motion missile 1)
            0x0022 => self.hmm0 = value, //HMM0 (horizontal motion missile 0)
            0x0021 => self.hmp1 = value, //HMP1 (horizontal motion player 1)
            0x0020 => self.hmp0 = value, //HMP0 (horizontal motion player 0)
            0x001F => self.enabl = value & 2, //ENABL (enable ball graphics)
            0x001E => self.enam1 = value & 2, //ENAM1 (enable missile 1 graphics)
            0x001D => self.enam0 = value & 2, //ENAM0 (enable missile 0 graphics)
            0x001C => {   //GRP1 (graphics player 1)
                self.grp1 = value;
                self.grp0a = self.grp0;
                self.enabla = self.enabl;
            }
            0x001B => {   //GRP0 (graphics player 0)
                self.grp0 = value;
                self.grp1a = self.grp1;
            }
            0x001A => self.audv[1] = value & 0xF, //AUDV1 (audio volume 1)
            0x0019 => self.audv[0] = value & 0xF, //AUDV0 (audio volume 0)
            0x0018 => { //AUDF1 (audio frequency 1)
                self.audf[1] = value & 0x1F;
            }
            0x0017 => { //AUDF0 (audio frequency 0)
                self.audf[0] = value & 0x1F; 
            }
            0x0016 => self.audc[1] = value & 0xF, //AUDC1 (audio control 1)
            0x0015 => self.audc[0] = value & 0xF, //AUDC0 (audio control 0)
            0x0014 => self.resbl = self.color_clock.max(68) + 3, //RESBL (reset ball)
            0x0013 => self.resm1 = self.color_clock.max(68) + 5, //RESM1 (reset missile 1)
            0x0012 => self.resm0 = self.color_clock.max(68) + 5, //RESM0 (reset missile 0)
            0x0011 => self.p1_cnt.set_delayed(0, 4), //RESP1 (reset player 1)
            0x0010 => self.p0_cnt.set_delayed(0, 4), //RESP0 (reset player 0)
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
            0x0003 => self.color_clock = 0, //RSYNC (reset horizontal sync counter)
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

struct Counter {
    value: u16,
    value_delayed: u16,
    assign_cnt: u8,
    cmp_cnt: u8,
    matched: u16
}

impl Counter {
    pub fn new(value: u16) -> Self {
        Self { value, value_delayed: 0, assign_cnt: 0, cmp_cnt: 0, matched: 0 }
    }

    pub fn value(&self) -> u16 {
        self.value
    }

    pub fn set(&mut self, value: u16) {
        self.value = value;
        self.value_delayed = value;
    }

    pub fn cmp(&mut self, value: u16) -> bool {
        if self.value == value {
            true
        } else {
            false
        }
    }

    pub fn set_delayed(&mut self, value: u16, delay: u8) {
        self.value_delayed = value;
        self.assign_cnt = delay + 1;
    }

    pub fn cmp_delayed(&mut self, value: u16, delay: u8) -> bool {
        if self.value == value {
            self.matched = value;
            self.cmp_cnt = delay;
        }

        if self.cmp_cnt != 0 {
            self.cmp_cnt -= 1;
        } else if self.matched == value {
            self.matched = 0;
            return true;
        }

        false
    }

    pub fn cycle(&mut self) {
        self.value += 1;

        if self.assign_cnt != 0 {
            self.assign_cnt -= 1;

            if self.assign_cnt == 0 {
                self.value = self.value_delayed;
            }
        }
    }
}

struct GraphicsCounter {
    value: u8,
    cnt: u8
}

impl GraphicsCounter {
    pub fn new() -> Self {
        Self { value: 0, cnt: 0 }
    }

    pub fn reset(&mut self) {
        self.value = 8;
        self.cnt = 0;
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn cycle(&mut self, clk_div: u8) -> u8 {
        self.cnt += 1;
        let value = self.value;

        if self.cnt == clk_div {
            self.cnt = 0;

            if self.value > 0 {
                self.value -= 1;
            }
        }

        value
    }
}

fn modular_add<T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::cmp::PartialOrd + From<u8> + Copy>(a: T, b: T, modulo: T) -> T {
    if a + b > modulo - T::from(1) {
        a + b - modulo
    } else {
        a + b
    }
}

fn modular_sub<T: std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::cmp::PartialOrd + From<u8> + Copy>(a: T, b: T, modulo: T) -> T {
    if b > a {
        modulo - (b - a)
    } else {
        a - b
    }
}
