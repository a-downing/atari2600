pub fn palette_rgb(color: u8) -> (u8, u8, u8) {
    match color & !0x01 {
        0x00 => (0, 0, 0),
        0x02 => (26, 26, 26),
        0x04 => (57, 57, 57),
        0x06 => (91, 91, 91),
        0x08 => (126, 126, 126),
        0x0A => (162, 162, 162),
        0x0C => (199, 199, 199),
        0x0E => (237, 237, 237),
        0x10 => (25, 2, 0),
        0x12 => (58, 31, 0),
        0x14 => (93, 65, 0),
        0x16 => (130, 100, 0),
        0x18 => (167, 136, 0),
        0x1A => (204, 173, 0),
        0x1C => (242, 210, 25),
        0x1E => (254, 250, 64),
        0x20 => (55, 0, 0),
        0x22 => (94, 8, 0),
        0x24 => (131, 39, 0),
        0x26 => (169, 73, 0),
        0x28 => (207, 108, 0),
        0x2A => (245, 143, 23),
        0x2C => (254, 180, 56),
        0x2E => (254, 223, 111),
        0x30 => (71, 0, 0),
        0x32 => (115, 0, 0),
        0x34 => (152, 19, 0),
        0x36 => (190, 50, 22),
        0x38 => (228, 83, 53),
        0x3A => (254, 118, 87),
        0x3C => (254, 156, 129),
        0x3E => (254, 198, 187),
        0x40 => (68, 0, 8),
        0x42 => (111, 0, 31),
        0x44 => (150, 6, 64),
        0x46 => (187, 36, 98),
        0x48 => (225, 69, 133),
        0x4A => (254, 103, 170),
        0x4C => (254, 140, 214),
        0x4E => (254, 183, 246),
        0x50 => (45, 0, 74),
        0x52 => (87, 0, 103),
        0x54 => (125, 5, 140),
        0x56 => (161, 34, 177),
        0x58 => (199, 67, 215),
        0x5A => (237, 101, 254),
        0x5C => (254, 138, 246),
        0x5E => (254, 181, 247),
        0x60 => (13, 0, 130),
        0x62 => (51, 0, 162),
        0x64 => (85, 15, 201),
        0x66 => (120, 45, 240),
        0x68 => (156, 78, 254),
        0x6A => (195, 114, 254),
        0x6C => (235, 152, 254),
        0x6E => (254, 192, 249),
        0x70 => (0, 0, 145),
        0x72 => (10, 5, 189),
        0x74 => (40, 34, 228),
        0x76 => (72, 66, 254),
        0x78 => (107, 100, 254),
        0x7A => (144, 138, 254),
        0x7C => (183, 176, 254),
        0x7E => (223, 216, 254),
        0x80 => (0, 0, 114),
        0x82 => (0, 28, 171),
        0x84 => (3, 60, 214),
        0x86 => (32, 94, 253),
        0x88 => (64, 129, 254),
        0x8A => (100, 166, 254),
        0x8C => (137, 206, 254),
        0x8E => (176, 246, 254),
        0x90 => (0, 16, 58),
        0x92 => (0, 49, 110),
        0x94 => (0, 85, 162),
        0x96 => (5, 121, 200),
        0x98 => (35, 157, 238),
        0x9A => (68, 194, 254),
        0x9C => (104, 233, 254),
        0x9E => (143, 254, 254),
        0xA0 => (0, 31, 2),
        0xA2 => (0, 67, 38),
        0xA4 => (0, 105, 87),
        0xA6 => (0, 141, 122),
        0xA8 => (27, 177, 158),
        0xAA => (59, 215, 195),
        0xAC => (93, 254, 233),
        0xAE => (134, 254, 254),
        0xB0 => (0, 36, 3),
        0xB2 => (0, 74, 5),
        0xB4 => (0, 112, 12),
        0xB6 => (9, 149, 43),
        0xB8 => (40, 186, 76),
        0xBA => (73, 224, 110),
        0xBC => (108, 254, 146),
        0xBE => (151, 254, 181),
        0xC0 => (0, 33, 2),
        0xC2 => (0, 70, 4),
        0xC4 => (8, 107, 0),
        0xC6 => (40, 144, 0),
        0xC8 => (73, 181, 9),
        0xCA => (107, 219, 40),
        0xCC => (143, 254, 73),
        0xCE => (187, 254, 105),
        0xD0 => (0, 21, 1),
        0xD2 => (16, 54, 0),
        0xD4 => (48, 89, 0),
        0xD6 => (83, 126, 0),
        0xD8 => (118, 163, 0),
        0xDA => (154, 200, 0),
        0xDC => (191, 238, 30),
        0xDE => (232, 254, 62),
        0xE0 => (26, 2, 0),
        0xE2 => (59, 31, 0),
        0xE4 => (94, 65, 0),
        0xE6 => (131, 100, 0),
        0xE8 => (168, 136, 0),
        0xEA => (206, 173, 0),
        0xEC => (244, 210, 24),
        0xEE => (254, 250, 64),
        0xF0 => (56, 0, 0),
        0xF2 => (95, 8, 0),
        0xF4 => (132, 39, 0),
        0xF6 => (170, 73, 0),
        0xF8 => (208, 107, 0),
        0xFA => (246, 143, 24),
        0xFC => (254, 180, 57),
        0xFE => (254, 223, 112),
        _ => panic!()
    }
}