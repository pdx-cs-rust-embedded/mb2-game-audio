#![no_std]
#![no_main]

use panic_rtt_target as _;
use cortex_m::asm;
use cortex_m_rt::entry;
use microbit::{Board, hal::gpio};
use rtt_target::rtt_init_print;

use mb2_game_audio::{GameAudio, Song, Note};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let speaker_pin = board.speaker_pin
        .into_push_pull_output(gpio::Level::Low)
        .degrade();
    let game_audio = GameAudio::new(board.TIMER0, board.PWM0, speaker_pin);
    loop {
        asm::wfe();
    }
}
