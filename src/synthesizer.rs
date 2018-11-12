//! Generate audio samples from code.
//!
//! The following code generates a 0.1s long, 16-bit, 440Hz sine_wave at a 44100Hz sample rate.
//! It then writes the generated samples into a 44100Hz WAV file at `out/sine.wav`.
//!
//! ```
//! use synthrs::wave::sine_wave;
//! use synthrs::writer::write_wav_file;
//! use synthrs::synthesizer::{quantize_samples, make_samples};
//!
//! write_wav_file(
//!     "out/sine.wav",
//!     44_100,
//!     &quantize_samples::<i16>(&make_samples(0.1, 44_100, sine_wave(440.0))),
//! ).expect("failed to write wav");
//! ```
//!
//! See: `examples/simple.rs`

use std::iter::Iterator;
use std::mem::size_of;

use num::traits::{Bounded, FromPrimitive, Num, Zero};
use num::Float;

use crate::errors::SynthrsError;
use crate::filter;
use crate::midi;
use crate::music;

/// Quantizes a `f64` sample into `T`.
/// Convert from [-1.0f64, 1.0] to take up full quantization range of type `T`.
/// Behaviour is not guaranteed for `T` where `T` is unsigned.
///
/// ```
/// use synthrs::synthesizer::quantize;
///
/// assert_eq!(quantize::<i8>(1.0f64), 127i8);
/// assert_eq!(quantize::<f32>(0.0f64), 0.0f32);
/// assert_eq!(quantize::<u8>(1.0f64), 127u8); // half of available quantization space of a u8 (255)
/// // quantize::<u8>(-1.0f64); // panics
/// ```
pub fn quantize<T>(input: f64) -> T
where
    T: Num + FromPrimitive + Bounded + Zero,
{
    let quantization_levels = 2.0.powf(size_of::<T>() as f64 * 8.0) - 1.0;
    // defaults to 0 on quantization failure for whatever reason
    T::from_f64(input * (quantization_levels / 2.0)).unwrap_or(T::zero())
}

/// Quantizes a `Vec<f64>` of samples into `Vec<T>`.
///
/// This creates a 16-bit `sine_wave` at 440Hz:
///
/// ```
/// use synthrs::wave::sine_wave;
/// use synthrs::synthesizer::{quantize_samples, make_samples};
///
/// quantize_samples::<i16>(&make_samples(1.0, 44_100, sine_wave(440.0)));
/// ```
pub fn quantize_samples<T>(input: &[f64]) -> Vec<T>
where
    T: Num + FromPrimitive + Bounded + Zero,
{
    input.iter().map(|s| quantize::<T>(*s)).collect()
}

/// Invokes the waveform function `f` at time `t` to return the amplitude at that time.
///
/// ```
/// use synthrs::synthesizer::generate;
/// use synthrs::wave::sine_wave;
///
/// let output = generate(1.0, &sine_wave(440.0));
/// ```
pub fn generate<F>(t: f64, f: &F) -> f64
where
    F: Fn(f64) -> f64,
{
    f.call((t,))
}

/// Given a generator waveform, returns a `Vec<f64>` of raw samples (not normalised or quantised)
///
/// `length` is in seconds
/// `sample_rate` is in hertz (eg `44_100`)
///
/// ```
/// use synthrs::synthesizer::make_samples;
/// use synthrs::wave;
///
/// let sine = make_samples(0.1, 44_100, |t| {
///     (t * 440.0 * 2.0 * 3.14159).sin()
/// });
///
/// let square = make_samples(0.1, 44_100, wave::square_wave(440.0));
/// ```
pub fn make_samples<F>(length: f64, sample_rate: usize, waveform: F) -> Vec<f64>
where
    F: Fn(f64) -> f64,
{
    let num_samples = (sample_rate as f64 * length).floor() as usize;
    let mut samples: Vec<f64> = Vec::with_capacity(num_samples);

    for i in 0usize..num_samples {
        let t = i as f64 / sample_rate as f64;
        samples.push(generate(t, &waveform));
    }

    samples
}

/// An iterator that generates samples of a waveform at a given sample rate
///
/// ```
/// use synthrs::synthesizer::SamplesIter;
/// use synthrs::wave::sine_wave;
///
/// let mut sine_iter = SamplesIter::new(44_100, Box::new(sine_wave(440.0)));
/// let samples = sine_iter.take(44_100).collect::<Vec<f64>>(); // take 1 second of samples
/// let _slice = samples.as_slice();
/// ```
pub struct SamplesIter {
    i: u64,
    sample_rate: u64,
    waveform: Box<Fn(f64) -> f64 + Send + 'static>,
}

impl SamplesIter {
    /// Returns an iterator that generates samples for the waveform at the given sample rate
    pub fn new(sample_rate: u64, waveform: Box<Fn(f64) -> f64 + Send + 'static>) -> SamplesIter {
        SamplesIter {
            i: 0,
            sample_rate: sample_rate,
            waveform: waveform,
        }
    }
}

impl Iterator for SamplesIter {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        let t = self.i as f64 / self.sample_rate as f64;
        self.i += 1;
        Some(self.waveform.call((t,)))
    }
}

/// Peak normalizes a `Vec<f64>` of samples such that the maximum and minimum amplitudes of the
/// `Vec<f64>` samples are within the range [-1.0, 1.0]
///
/// ```
/// use synthrs::synthesizer::{make_samples, peak_normalize};
/// use synthrs::wave;
///
/// let samples = make_samples(0.1, 44_100, wave::sine_wave(440.0));
/// let normalized = peak_normalize(&samples);
/// ```
pub fn peak_normalize(samples: &[f64]) -> Vec<f64> {
    let peak = samples
        .iter()
        .fold(0.0f64, |acc, &sample| acc.max(sample).max(-sample));

    samples.iter().map(|&sample| sample / peak).collect()
}

// This is really awful, is there a more elegant way to do this?
/// Generates samples from a MIDI file
///
/// ```
/// use synthrs::synthesizer::make_samples_from_midi_file;
/// use synthrs::wave;
///
/// let samples = make_samples_from_midi_file(
///     wave::sine_wave,
///     44_100,
///     false,
///     "tests/assets/test.mid",
/// ).unwrap();
/// ```
pub fn make_samples_from_midi_file<F1, F2>(
    instrument: F1,
    sample_rate: usize,
    use_envelope: bool,
    path: &str,
) -> Result<Vec<f64>, SynthrsError>
where
    F1: Fn(f64) -> F2,
    F2: Fn(f64) -> f64,
{
    let song = midi::read_midi_file(path)?;
    make_samples_from_midi(instrument, sample_rate, use_envelope, song)
}

// This is really awful, is there a more elegant way to do this?
/// Generates samples from a MIDI file. Supports only one instrument. Instrument can be any generator.
///
/// `instrument` is the waveform generator
/// `use_envelope` decide whether to use a basic attack/decay envelope when generating samples
///
/// ```
/// use synthrs::synthesizer::make_samples_from_midi;
/// use synthrs::midi;
/// use synthrs::wave;
///
/// let song = midi::read_midi_file("tests/assets/test.mid").unwrap();
///
/// let samples = make_samples_from_midi(
///     |frequency: f64| wave::bell(frequency, 0.003, 0.5),
///     44_100,
///     false,
///     song.clone(),
/// ).unwrap();
///
/// let samples = make_samples_from_midi(
///     wave::sine_wave,
///     44_100,
///     true,
///     song.clone(),
/// ).unwrap();
/// ```
pub fn make_samples_from_midi<F1, F2>(
    instrument: F1,
    sample_rate: usize,
    use_envelope: bool,
    song: midi::MidiSong,
) -> Result<Vec<f64>, SynthrsError>
where
    F1: Fn(f64) -> F2,
    F2: Fn(f64) -> f64,
{
    let length = (60.0 * song.max_time as f64) / (song.bpm * song.time_unit as f64);

    let mut notes_on_for_ticks: Vec<Vec<(u8, u8, usize, usize, usize)>> = Vec::new();
    for _ in 0..song.max_time {
        let notes_on_for_tick: Vec<(u8, u8, usize, usize, usize)> = Vec::new();
        notes_on_for_ticks.push(notes_on_for_tick);
    }

    for track in &song.tracks {
        for i in 0..track.events.len() {
            let event = track.events[i];
            if event.event_type == midi::EventType::NoteOn {
                let start_tick = event.time;
                let note = event.value1;
                let velocity = event.value2.unwrap_or(0);

                let mut end_tick = song.max_time;
                for j in i..track.events.len() {
                    let event_cursor = track.events[j];

                    if event_cursor.value1 == note && event_cursor.is_note_terminating() {
                        end_tick = event_cursor.time;
                        break;
                    }
                }

                for (i, on_notes) in notes_on_for_ticks
                    .iter_mut()
                    .enumerate()
                    .take(end_tick)
                    .skip(start_tick)
                {
                    let ticks_left = end_tick - i;
                    on_notes.push((note as u8, velocity as u8, start_tick, i, ticks_left));
                }
            }
        }
    }

    let midi_frequency_function = |t: f64| -> f64 {
        let tick = (t * song.bpm * song.time_unit as f64 / 60.0) as usize;
        let mut out = 0.0;

        if tick < notes_on_for_ticks.len() {
            for &(note, velocity, start_tick, _ticks_elasped, _ticks_left) in
                &notes_on_for_ticks[tick]
            {
                let frequency = music::note_midi(440.0, note as usize);
                // TODO: split loudness into a util module
                let loudness = (6.908 * (f64::from(velocity) / 255.0)).exp() / 1000.0;
                out += loudness * (instrument)(frequency)(t);

                if use_envelope {
                    let start_t =
                        start_tick as f64 * 60.0 / song.bpm as f64 / song.time_unit as f64;

                    // TODO: make this an option
                    let attack = 0.01;
                    let decay = 1.0;
                    let relative_t = t - start_t;
                    out *= filter::envelope(relative_t, attack, decay);

                    // TODO: make this an option, since it's awful for sine wave
                    // // Reduce clicks when changing notes
                    // // Ideally phase shift, but this is a simple solution
                    // let fade_ticks = 20;

                    // if ticks_elasped < fade_ticks {
                    //     let ratio = ticks_elasped as f64 / fade_ticks as f64;
                    //     out *= ratio * ratio;
                    // } else if ticks_left < fade_ticks {
                    //     let ratio = ticks_left as f64 / fade_ticks as f64;
                    //     out *= ratio * ratio;
                    // }
                }
            }
        }

        out
    };

    let num_samples = (sample_rate as f64 * length).floor() as usize;
    let mut samples: Vec<f64> = Vec::with_capacity(num_samples);

    for i in 0usize..num_samples {
        let t = i as f64 / sample_rate as f64;
        samples.push(midi_frequency_function(t));
    }

    Ok(peak_normalize(&samples))
}

#[cfg(test)]
mod tests {
    use std::i16;
    use std::i8;

    use super::*;
    use crate::wave::sine_wave;

    #[test]
    fn it_peak_normalizes() {
        let input_negative = vec![-2.0f64, 1.0, -1.0];
        let output_negative = peak_normalize(&input_negative);
        assert_eq!(output_negative, vec![-1.0f64, 0.5, -0.5]);

        let input_positive = vec![2.0f64, 1.0, -1.0];
        let output_positive = peak_normalize(&input_positive);
        assert_eq!(output_positive, vec![1.0f64, 0.5, -0.5])
    }

    #[test]
    fn it_quantizes() {
        assert_eq!(i8::MAX, quantize::<i8>(1.0));
        // assert_eq!(i8::MIN + 1, quantize::<i8>(-1.0)); // Bad quantization behaviour?
        assert_eq!(i16::MAX, quantize::<i16>(1.0));
        assert_eq!(0.0f32, quantize::<f32>(0.0));
        // assert_eq!(u8::MAX, quantize::<u8>(1.0)); // TODO: Make quantization work for unsigned types
    }

    #[test]
    fn test_samples_iterator() {
        let mut iter = SamplesIter::new(1, Box::new(sine_wave(3.1415)));
        assert_eq!(iter.next().unwrap(), 0.0);
        assert_eq!(iter.next().unwrap(), 0.7764865126870779);
        assert_eq!(iter.next().unwrap(), 0.9785809043254725);
    }

    #[test]
    fn test_make_samples() {
        let waveform = sine_wave(3.1415);
        let samples = make_samples(3.0, 1, waveform);
        assert_eq!(vec![0.0, 0.7764865126870779, 0.9785809043254725], samples);
    }
}
