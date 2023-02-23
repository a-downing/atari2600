use std::{thread, time::Duration};

use atari2600::{atari2600::{Atari2600}, tia, riot::{Player, JoystickDirection}};
use sdl2::{event::Event, pixels::Color, rect::Point, keyboard::Keycode, audio::AudioSpecDesired};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let rom = std::fs::read(&args[1]).unwrap();

    let atari = Atari2600::new(rom);

    let mut cpu = atari2600::MOS6502::new(atari);
    cpu.reset();
    cpu.get_bus().riot.switch_color(true);
    cpu.get_bus().riot.switch_select(true);
    cpu.get_bus().riot.switch_reset(true);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    const SCALE: u16 = 3;

    let spec = AudioSpecDesired{ freq: Some(44100), channels: Some(1), samples: Some(1024) };
    let audio_device = audio_subsystem.open_queue::<u8, _>(None, &spec).unwrap();
    audio_device.resume();

    let window = video_subsystem.window("atari2600", (tia::CLOCKS_PER_SCANLINE * SCALE) as u32, (tia::NUM_SCANLINES * SCALE) as u32)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");
    
    let mut canvas = window.into_canvas().build().expect("could not make a canvas");
    canvas.set_scale(SCALE as f32, SCALE as f32).unwrap();

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let player = Player::Zero;

    let mut samples = Vec::<u8>::new();
    const CLOCKS_PER_SAMPLE: u16 = 85; // 3.75Mhz / 44100Hz
    let mut clocks_consumed = 0;
    let mut sample = 0.0;

    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main_loop,
                Event::KeyDown { timestamp, window_id, keycode, scancode, keymod, repeat } => match keycode {
                    Some(keycode) => match keycode {
                        Keycode::Right => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Right, true),
                        Keycode::Left => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Left, true),
                        Keycode::Down => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Down, true),
                        Keycode::Up => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Up, true),
                        Keycode::Space => cpu.get_bus().tia.input4(0x00),
                        _ => ()
                    },
                    None => (),
                }
                Event::KeyUp { timestamp, window_id, keycode, scancode, keymod, repeat } => match keycode {
                    Some(keycode) => match keycode {
                        Keycode::Right => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Right, false),
                        Keycode::Left => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Left, false),
                        Keycode::Down => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Down, false),
                        Keycode::Up => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Up, false),
                        Keycode::Space => cpu.get_bus().tia.input4(0x80),
                        _ => ()
                    },
                    None => (),
                }
                _ => ()
            }
        }

        for _ in 0..1000 {
            let wsync = {
                let atari = cpu.get_bus();
                atari.tia.wsync()
            };

            {
                let atari = cpu.get_bus();

                for _ in 0..3 {
                    atari.tia.cycle();
                }

                atari.riot.cycle();

                if atari.tia.draw() {
                    atari.tia.drew();
                    canvas.set_draw_color(Color::BLACK);
                    canvas.clear();
                    canvas.set_draw_color(Color::WHITE);

                    for x in 0..tia::CLOCKS_PER_SCANLINE {
                        for y in 0..tia::NUM_SCANLINES {
                            let index = y * tia::CLOCKS_PER_SCANLINE + x;
                            let pixel = atari.tia.frame[index as usize];

                            if pixel != 0 {
                                canvas.set_draw_color(atari2600::palette_rgb(pixel));
                                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
                            }
                        }
                    }

                    canvas.present();

                    loop {
                        if atari.tia.audio_ch1.len() < 2 {
                            break;
                        }

                        let mut s0 = atari.tia.audio_ch1[0];
                        let s1 = atari.tia.audio_ch1[1];

                        while clocks_consumed < CLOCKS_PER_SAMPLE {
                            let clocks = s1.cycles.wrapping_sub(s0.cycles);

                            //println!("clocks: {}", clocks);
                            let clocks_needed = CLOCKS_PER_SAMPLE - clocks_consumed;
                            
                            if clocks >= clocks_needed {
                                s0.cycles = s0.cycles.wrapping_add(clocks_needed);
                                atari.tia.audio_ch1[0] = s0;
                                clocks_consumed = 0;
                                sample += s0.value as f32 * clocks_needed as f32;
                                samples.push((sample / CLOCKS_PER_SAMPLE as f32) as u8);
                                sample = 0.0;
                            } else {
                                clocks_consumed += clocks;
                                sample += s0.value as f32 * clocks as f32;
                                atari.tia.audio_ch1.pop_front();
                                break;
                            }
                        }
                    }

                    audio_device.queue_audio(&samples).unwrap();
                    samples.clear();

                    while audio_device.size() > 1024 {
                        thread::sleep(Duration::from_micros(1));
                    }
                }
            }

            cpu.cycle(!wsync);
        }
    }
}
