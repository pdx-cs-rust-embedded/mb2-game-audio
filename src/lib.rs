use embedded_hal::{digital::{OutputPin, PushPull}, timer};
use nrf52833_hal::Pwm;

pub struct GameAudio<T> {
    timer: T,
    pwm: Pwm,
    speaker: Pin<Output<PushPull>>,
}

pub struct Note {
    frequency: u16,
    duration: u16,
}    

impl<T: timer::Instance> GameAudio<T> {
    pub fn new(timer: T, pwm: Pwm, speaker: OutputPin<PushPull>) -> Self {
        Self {
            timer,
            pwm,
            speaker,
        }
    }

    pub fn play(&mut self, song: &[Note]) {
        todo!()
    }
}
