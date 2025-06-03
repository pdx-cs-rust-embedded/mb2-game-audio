#![allow(unused)]

/*!
## Game audio library crate for the BBC micro:bit v2 (MB2)

This crate currently provides basic support for starting and
stopping a background song in a MB2 program.
*/

use embedded_hal::digital::OutputPin;
use nrf52833_hal::{pwm, timer};

/// A note in the song.
pub struct Note {
    /// Frequency of note in Hz.
    frequency: u16,
    /// Duration of note in ms.
    duration: u16,
}

pub struct Song<'a> {
    notes: &'a [Note],
    position: usize,
}

impl<'a> Song<'a> {
    pub fn new(notes: &'a [Note]) -> Self {
        Self { notes, position: 0 }
    }

    pub fn restart(&mut self) {
        self.position = 0;
    }
}

pub struct GameAudio<'a, T, P, S> {
    timer: T,
    pwm: P,
    speaker: S,
    song: Option<Song<'a>>,
}

impl<'a, T, P, S> GameAudio<'a, T, pwm::Pwm<P>, S>
where
    T: timer::Instance,
    P: pwm::Instance,
    S: OutputPin,
{
    pub fn new(timer: T, pwm: pwm::Pwm<P>, speaker: S) -> Self {
        Self {
            timer,
            pwm,
            speaker,
            song: None,
        }
    }

    pub fn play(&mut self, song: Song<'a>) {
        self.song = Some(song);
    }

    pub fn stop(&mut self) {
        self.song = None;
    }
}
