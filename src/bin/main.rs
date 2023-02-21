use atari2600::{atari2600::{Atari2600}, tia, riot::{Player, JoystickDirection}};
use sdl2::{event::Event, pixels::Color, rect::Point, keyboard::Keycode};

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
    const SCALE: u16 = 3;

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
                }
            }

            cpu.cycle(!wsync);
        }
    }
}
