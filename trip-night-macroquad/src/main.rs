use game_clock::Time;
use macroquad::prelude::*;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;
use trip_night_core::machine::Machine;
use trip_night_core::screen::PixelState;

const PIXEL_SIZE: f32 = 16.0;
const CLOCK_FREQUENCY: usize = 700;
const REFRESH_RATE: f64 = 30.0;

#[macroquad::main("Trip Night VM")]
async fn main() {
    let mut game_code = Vec::with_capacity(4096);
    BufReader::new(File::open("ibm_logo.ch8").unwrap())
        .read_to_end(&mut game_code)
        .unwrap();

    let standard_instruction_set = trip_night_instruction::make_standard_set();
    let mut machine = Machine::new(&game_code, standard_instruction_set, CLOCK_FREQUENCY);

    let mut time = Time::default();
    time.set_fixed_time(Duration::from_secs_f64(1.0 / REFRESH_RATE));

    let step = 1.0 / CLOCK_FREQUENCY as f64;

    clear_background(BLACK);
    next_frame().await;

    loop {
        machine.cycle();

        if machine.is_beeping() {
            println!("beep!");
        }

        while time.step_fixed_update() {
            if is_quit_requested() {
                break;
            }

            clear_background(BLACK);

            for (x, y, state) in machine.screen().iter() {
                match state {
                    PixelState::Set => {
                        let x = (x as f32) * PIXEL_SIZE;
                        let y = (y as f32) * PIXEL_SIZE;
                        draw_rectangle(x, y, PIXEL_SIZE, PIXEL_SIZE, WHITE);
                    }
                    PixelState::Unset => {}
                }
            }

            next_frame().await;
        }

        time.advance_frame(Duration::from_secs_f64(step));
    }
}

// NOTE:
// Standard CHIP-8 keypad:
// 1 | 2 | 3 | C
// - | - | - | -
// 4 | 5 | 6 | D
// - | - | - | -
// 7 | 8 | 9 | E
// - | - | - | -
// A | 0 | B | F
// Generally mapped as follow on a modern QWERTY keyboard:
// 1 | 2 | 3 | 4
// - | - | - | -
// Q | W | E | R
// - | - | - | -
// A | S | D | F
// - | - | - | -
// Z | X | C | V
// Note: use scancodes so it’s layout-agnostic
