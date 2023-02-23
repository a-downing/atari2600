use std::io::{stdout, Write};

fn main() {
    let mut poly4: u16 = 1;
    let mut poly5: u16 = 2;
    let mut poly9: u16 = 1;
    let mut output: bool = false;

    loop {
        let f5 = ((poly5 >> 2) ^ (poly5 >> 0)) & 1;
        poly5 = (f5 << 4) | ((poly5 & 0b11111) >> 1);

        let out4 = poly4 & 1;

        //if out5 != 0 {
        //    let f4 = ((poly4 >> 1) ^ (poly4 >> 0)) & 1;
        //    poly4 = (f4 << 3) | ((poly4 & 0b1111) >> 1);
        //}

        let out9 = poly9 & 1;
        let f9 = ((poly9 >> 4) ^ (poly9 >> 0)) & 1;
        poly9 = (f9 << 8) | ((poly9 & 0b111111111) >> 1);


        //println!("poly4: {} {:04b} {}", out4, poly4, poly4);
        //println!("poly5: {} {:05b} {}", poly5 & 1, poly5 & 0b11, poly5 & 0b11);
        //println!("poly9: {} {:09b} {}", out9, poly9, poly9);

        if poly5 == 1 {
            output = true;
        }
        
        if poly5 == 3 {
            output = false;
        }

        stdout().write_all(&[1, if output { 255 } else { 0 }]).unwrap();
    }
}