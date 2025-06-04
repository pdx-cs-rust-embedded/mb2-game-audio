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

/// Maintain a sequence of notes with a current position.
/// Playback will move forward through the notes, looping.
pub struct Song<'a> {
    /// Sequence of notes.
    notes: &'a [Note],
    /// Current position in sequence.
    position: usize,
}

impl<'a> Song<'a> {
    /// Make a new song from a note sequence.
    pub fn new(notes: &'a [Note]) -> Self {
        Self { notes, position: 0 }
    }

    /// Reset playback to the beginning of the song.
    pub fn restart(&mut self) {
        self.position = 0;
    }
}

/// Hardware being used for game audio,
/// together with the song being played if any.
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
    /// Accumulate the needed hardware to play.
    pub fn new(timer: T, pwm: pwm::Pwm<P>, speaker: S) -> Self {
        Self {
            timer,
            pwm,
            speaker,
            song: None,
        }
    }

    /// Start a song playing. Returns any previously-playing song.
    pub fn play(&mut self, song: Song<'a>) -> Option<Song<'a>> {
        self.song.replace(song)
    }

    /// Stop song playback. Returns the playing song in current state.
    pub fn stop(&mut self) -> Option<Song<'a>> {
        self.song.take()
    }
}
