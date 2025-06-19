#![no_std]
#![no_main]

use panic_rtt_target as _;
use cortex_m::asm;
use cortex_m_rt::entry;
use embedded_hal::delay::DelayNs;
use microbit::{
    Board,
    hal::{
        gpio,
        pwm,
        timer,
        pac::{self, TIMER0, PWM0, interrupt},
    }
};
use rtt_target::rtt_init_print;

use mb2_game_audio::{GameAudio, Song, Note};
use critical_section_lock_mut::LockMut;

type Timer0 = timer::Timer<TIMER0>;
type Pwm0 = pwm::Pwm<PWM0>;
static GAME_AUDIO: LockMut<GameAudio<'static, Timer0, Pwm0>> = LockMut::new();

static SONG: &[Note] = &[
    Note::rest(500),
    Note::note(68, 250, 1),
    Note::note(69, 250, 2),
    Note::note(68, 250, 3),
    Note::note(66, 500, 1),
];

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let speaker_pin = board.speaker_pin
        .into_push_pull_output(gpio::Level::Low)
        .degrade();
    let game_audio = GameAudio::new(board.TIMER0, board.PWM0, speaker_pin);
    let mut timer = timer::Timer::new(board.TIMER1);
    let song = Song::new(SONG);
    GAME_AUDIO.init(game_audio);

    unsafe { pac::NVIC::unmask(pac::Interrupt::TIMER0); }
    pac::NVIC::unpend(pac::Interrupt::TIMER0);

    GAME_AUDIO.with_lock(|ga| { ga.play(song); });
    timer.delay_ms(10_000);
    GAME_AUDIO.with_lock(|ga| { ga.stop(); });

    loop {
        asm::wfe();
    }
}

#[interrupt]
fn TIMER0() {
    GAME_AUDIO.with_lock(|ga| ga.handle_interrupt());
}
