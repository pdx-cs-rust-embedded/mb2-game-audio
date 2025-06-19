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
    // MIDI key of note.
    key: u8,
    // Volume of note in range 0..=6.
    volume: u8,
    // Duration of note in ms.
    duration: u16,
}

impl Note {
    /// A note with the given MIDI key number in the range 0
    /// through 127, given duration in milliseconds, and
    /// given volume in the range 0 through 6.
    #[allow(clippy::self_named_constructors)]
    pub const fn note(key: u8, duration: u16, volume: u8) -> Self {
        assert!(key < 128);
        assert!(volume <= 6);
        Self { key, volume, duration }
    }

    /// A silence of the given duration.
    pub const fn rest(duration: u16) -> Self {
        Self { key: 0, volume: 0, duration }
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
        // See the comment elsewhere about speaker volume.
        speaker.set_high().unwrap();

        let pwm = pwm::Pwm::new(pwm);
        pwm.disable();
        pwm
            .set_output_pin(pwm::Channel::C0, speaker)
            .set_prescaler(pwm::Prescaler::Div16)
            .set_counter_mode(pwm::CounterMode::UpAndDown);

        let mut timer = timer::Timer::new(timer);
        timer.enable_interrupt();

        Self {
            timer,
            pwm,
            song: None,
        }
    }

    /// Start a song playing. Returns any previously-playing
    /// song in its current state.
    pub fn play(&mut self, song: Song<'a>) -> Option<Song<'a>> {
        let result = self.song.replace(song);
        self.handle_interrupt();
        result
    }

    /// Stop song playback. Returns any currently-playing
    /// song in its current state.
    pub fn stop(&mut self) -> Option<Song<'a>> {
        let result = self.song.take();
        self.handle_interrupt();
        result
    }

    pub fn handle_interrupt(&mut self) {
        #[cfg(feature = "trace")]
        rprintln!("i");
        let mut silent = true;
        if let Some(ref mut song) = self.song {
            let p = song.position;
            let note = song.notes[p];
            song.position = (p + 1) % song.notes.len();

            if note.volume > 0 {
                let f = keytones::key_to_frequency(note.key).round() as u32;
                self.pwm.set_period(time::Hertz(f));

                let v = (1 << note.volume) + 64;
                let d = self.pwm.max_duty() as u32 * v / 256;
                self.pwm.set_duty_on_common(d as u16);

                // Make sure the PWM is enabled.
                self.pwm.enable();

                #[cfg(feature = "trace")]
                rprintln!("n {} ({}) {} ({})", note.key, f, note.volume, v);
                silent = false;
            }

            #[cfg(feature = "trace")]
            rprintln!("{}", note.duration);
            self.timer.enable_interrupt();
            self.timer.start(note.duration as u32 * 1000);
        }
        if silent {
            #[cfg(feature = "trace")]
            rprintln!("r");

            /* We control speaker volume by controlling PWM
            duty cycle. If the speaker power capacitor is
            not kept close to discharged then the first
            audible signal will be at abnormally high volume
            regardless of the duty cycle setting, since it
            will be drawing a full 3.3V from the charged
            capacitor rather than the smaller voltage
            normally available there while free-running.

            By disabling PWM and holding the speaker pin
            high during silences we keep the speaker
            capacitor near its nominal discharged state. */
            self.pwm.disable();
            let mut speaker_pin = self.pwm
                .clear_output_pin(pwm::Channel::C0)
                .unwrap();
            speaker_pin.set_high().unwrap();
            self.pwm.set_output_pin(pwm::Channel::C0, speaker_pin);
        }
        self.timer.reset_event();
    }
}
