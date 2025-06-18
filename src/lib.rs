/*!
## Game audio library crate for the BBC micro:bit v2 (MB2)

This crate currently provides basic support for starting and
stopping a background song in a MB2 program.
*/

#![no_std]

use embedded_hal::digital::OutputPin;
use keytones::{self, Float};
use nrf52833_hal::{gpio, pwm, time, timer};
#[cfg(feature = "trace")]
use rtt_target::rprintln;

/// A note in the song.
#[derive(Clone, Copy)]
pub struct Note {
    /// MIDI key of note, or rest.
    key: Option<u8>,
    /// Duration of note in ms.
    duration: u16,
}

impl Note {
    #[allow(clippy::self_named_constructors)]
    pub const fn note(key: u8, duration: u16) -> Self {
        Self {
            key: Some(key),
            duration,
        }
    }

    pub const fn rest(duration: u16) -> Self {
        Self {
            key: None,
            duration,
        }
    }
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
pub struct GameAudio<'a, T, P> {
    timer: T,
    pwm: P,
    song: Option<Song<'a>>,
}

type SpeakerPin = gpio::Pin<gpio::Output<gpio::PushPull>>;

impl<'a, T, P> GameAudio<'a, timer::Timer<T>, pwm::Pwm<P>>
where
    T: timer::Instance,
    P: pwm::Instance,
{
    /// Accumulate the needed hardware to play. Set up the
    /// hardware according to purpose.
    pub fn new(timer: T, pwm: P, mut speaker: SpeakerPin) -> Self {
        let pwm = pwm::Pwm::new(pwm);
        speaker.set_low().unwrap();
        pwm.set_output_pin(pwm::Channel::C0, speaker);
        pwm.disable();

        let timer = timer::Timer::new(timer);

        Self {
            timer,
            pwm,
            song: None,
        }
    }

    /// Start a song playing. Returns any previously-playing song.
    pub fn play(&mut self, song: Song<'a>) -> Option<Song<'a>> {
        let result = self.song.replace(song);
        self.handle_interrupt();
        result
    }

    /// Stop song playback. Returns the playing song in current state.
    pub fn stop(&mut self) -> Option<Song<'a>> {
        self.pwm.disable();
        self.timer.disable_interrupt();
        self.song.take()
    }

    pub fn handle_interrupt(&mut self) {
        #[cfg(feature = "trace")]
        rprintln!("i");
        if let Some(ref mut song) = self.song {
            let p = song.position;
            let note = song.notes[p];
            song.position = (p + 1) % song.notes.len();

            if let Some(k) = note.key {
                let f = keytones::key_to_frequency(k).round() as u32;
                #[cfg(feature = "trace")]
                rprintln!("{}", f);
                self.pwm
                    .set_prescaler(pwm::Prescaler::Div16)
                    .set_counter_mode(pwm::CounterMode::UpAndDown)
                    .set_period(time::Hertz(f))
                    .set_duty_on_common(self.pwm.max_duty() / 2);
                self.pwm.enable();
            } else {
                #[cfg(feature = "trace")]
                rprintln!("r");
                self.pwm.disable();
            }

            #[cfg(feature = "trace")]
            rprintln!("{}", note.duration);
            self.timer.enable_interrupt();
            self.timer.start(note.duration as u32 * 1000);
        }
        self.timer.reset_event();
    }
}
