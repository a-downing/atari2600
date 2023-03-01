use std::{thread, time::{Duration, Instant}, io::Write};

use atari2600::{atari2600::{Atari2600}, tia, riot::{Player, JoystickDirection}, AudioConverter};
use sdl2::{event::Event, pixels::{Color, PixelFormatEnum}, keyboard::Keycode, audio::AudioSpecDesired, render::TextureAccess};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let rom = std::fs::read(&args[1]).unwrap();

    let atari = Atari2600::new(rom);

    let mut cpu = atari2600::MOS6502::new(atari);
    cpu.reset();
    cpu.get_bus().riot.switch_color(true);
    //cpu.get_bus().riot.switch_select(true);
    //cpu.get_bus().riot.switch_reset(true);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    const SCALE: u16 = 3;

    let spec = AudioSpecDesired{ freq: Some(44100), channels: Some(2), samples: Some(512) };
    let audio_device = audio_subsystem.open_queue::<u8, _>(None, &spec).unwrap();
    audio_device.resume();

    let window = video_subsystem.window("atari2600", (tia::CLOCKS_PER_SCANLINE * SCALE) as u32, (tia::NUM_SCANLINES * SCALE) as u32)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");
    
    let mut canvas = window.into_canvas().accelerated().build().expect("could not make a canvas");
    canvas.set_scale(SCALE as f32, SCALE as f32).unwrap();

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, tia::CLOCKS_PER_SCANLINE as u32, tia::NUM_SCANLINES as u32).unwrap();
    let mut pixels = [0u8; tia::NUM_SCANLINES as usize * tia::CLOCKS_PER_SCANLINE as usize * 3];

    let mut event_pump = sdl_context.event_pump().unwrap();
    let player = Player::Zero;

    let mut audio_converter = AudioConverter::new(85);
    let mut frame_num = 0;
    let start_time = Instant::now();
    

    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'main_loop,
                Event::KeyDown { timestamp: _, window_id: _, keycode, scancode: _, keymod: _, repeat: _ } => match keycode {
                    Some(keycode) => match keycode {
                        Keycode::Right => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Right, true),
                        Keycode::Left => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Left, true),
                        Keycode::Down => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Down, true),
                        Keycode::Up => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Up, true),
                        Keycode::Space => cpu.get_bus().tia.input4(0x00),
                        Keycode::S => cpu.get_bus().riot.switch_select(false),
                        Keycode::R => cpu.get_bus().riot.switch_reset(false),
                        _ => ()
                    },
                    None => (),
                }
                Event::KeyUp { timestamp: _, window_id: _, keycode, scancode: _, keymod: _, repeat: _ } => match keycode {
                    Some(keycode) => match keycode {
                        Keycode::Right => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Right, false),
                        Keycode::Left => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Left, false),
                        Keycode::Down => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Down, false),
                        Keycode::Up => cpu.get_bus().riot.switch_joystick(player, JoystickDirection::Up, false),
                        Keycode::Space => cpu.get_bus().tia.input4(0x80),
                        Keycode::S => cpu.get_bus().riot.switch_select(true),
                        Keycode::R => cpu.get_bus().riot.switch_reset(true),
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

                    let t = Instant::now() - start_time;
                    
                    if frame_num % 10 == 0 {
                        println!("fps: {}", frame_num as f32 / t.as_secs_f32());
                    }

                    for x in 0..tia::CLOCKS_PER_SCANLINE as usize {
                        for y in 0..tia::NUM_SCANLINES as usize {
                            let index = y * tia::CLOCKS_PER_SCANLINE as usize + x;
                            let rgb = atari2600::palette_rgb(atari.tia.frame[index]);

                            pixels[index * 3 + 0] = rgb.0;
                            pixels[index * 3 + 1] = rgb.1;
                            pixels[index * 3 + 2] = rgb.2;

                        }
                    }

                    texture.update(None, &pixels, 3 * tia::CLOCKS_PER_SCANLINE as usize).unwrap();
                    canvas.copy(&texture, None, None).unwrap();
                    canvas.present();

                    if audio_device.size() == 0 {
                        println!("audio buffer underrun");
                    }

                    let samples = audio_converter.convert(&mut atari.tia.audio);
                    audio_device.queue_audio(samples).unwrap();

                    while audio_device.size() > 2048 {
                        thread::sleep(Duration::from_micros(1));
                    }

                    //std::io::stdout().write_all(samples.as_ref()).unwrap();

                    frame_num += 1;
                }
            }

            cpu.cycle(!wsync);
        }
    }
}