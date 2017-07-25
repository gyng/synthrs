//! The following code generates a 1s long, 16-bit, 440Hz sinewave at a 44100Hz sample rate.
//! It then writes the generated samples into a 44100Hz WAV file at `out/sine.wav`.
//!
//! ```ignore
//! use synthrs::wave::SineWave;
//! use synthrs::writer::write_wav;
//! use synthrs::synthesizer::{quantize_samples, make_samples};
//!
//! write_wav("out/sine.wav", 44100,
//!     quantize_samples::<i16>(make_samples(1.0, 44100, SineWave(440.0)))
//! ).ok().expect("failed");
//! ```
//!
//! See: `examples/simple.rs`

use std::mem::size_of;

use num::Float;
use num::traits::{Bounded, FromPrimitive, Num};

use filter;
use music;
use midi;
use wave;

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
/// assert_eq!(quantize::<u8>(-1.0f64), 129u8); // unexpected behaviour
/// ```
pub fn quantize<T>(input: f64) -> T
where
    T: Num + FromPrimitive + Bounded,
{
    let quantization_levels = 2.0.powf(size_of::<T>() as f64 * 8.0) - 1.0;
    T::from_f64(input * (quantization_levels / 2.0)).expect("failed to quantize to given type")
}

/// Quantizes a `Vec<f64>` of samples into `Vec<T>`.
///
/// This creates a 16-bit `SineWave` at 440Hz:
///
/// ```
/// use synthrs::wave::SineWave;
/// use synthrs::synthesizer::{quantize_samples, make_samples};
///
/// quantize_samples::<i16>(&make_samples(1.0, 44100, SineWave(440.0)));
/// ```
pub fn quantize_samples<T>(input: &[f64]) -> Vec<T>
where
    T: Num + FromPrimitive + Bounded,
{
    input.iter().map(|s| quantize::<T>(*s)).collect()
}

/// Invokes the waveform function `f` at time `t` to return the amplitude at that time.
pub fn generate<F>(t: f64, f: &F) -> f64
where
    F: Fn(f64) -> f64,
{
    f.call((t,))
}

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

/// Peak normalizes a `Vec<f64>` of samples such that the maximum and minimum amplitudes of the
/// `Vec<f64>` samples are within the range [-1.0, 1.0]
pub fn peak_normalize(samples: &[f64]) -> Vec<f64> {
    let peak = samples.iter().fold(0.0f64, |acc, &sample| {
        acc.max(sample).max(-sample)
    });

    samples.iter().map(|&sample| sample / peak).collect()
}

// This is really awful, is there a more elegant way to do this?
// TODO: Make the instrument a parameter (perhaps using an Instrument trait?)
pub fn make_samples_from_midi(sample_rate: usize, filename: &str) -> Vec<f64> {
    let song = midi::read_midi(filename).unwrap();
    let length = (60.0 * song.max_time as f64) / (song.bpm * song.time_unit as f64);

    let mut notes_on_for_ticks: Vec<Vec<(u8, u8, usize)>> = Vec::new();
    for _ in 0..song.max_time {
        let notes_on_for_tick: Vec<(u8, u8, usize)> = Vec::new();
        notes_on_for_ticks.push(notes_on_for_tick);
    }

    for track in &song.tracks {
        for i in 0..track.events.len() {
            let event = track.events[i];
            if event.event_type == midi::EventType::NoteOn {
                let start_tick = event.time;
                let note = event.value1;
                let velocity = event.value2.unwrap();

                let mut end_tick = song.max_time;
                for j in i..track.events.len() {
                    let event_cursor = track.events[j];

                    if event_cursor.value1 == note && event_cursor.is_note_terminating() {
                        end_tick = event_cursor.time;
                        break;
                    }
                }

                for on_notes in notes_on_for_ticks.iter_mut().take(end_tick).skip(
                    start_tick,
                )
                {
                    on_notes.push((note as u8, velocity as u8, start_tick));
                }
            }
        }
    }

    let midi_frequency_function = |t: f64| -> f64 {
        let tick = (t * song.bpm * song.time_unit as f64 / 60.0) as usize;
        let mut out = 0.0;

        if tick < notes_on_for_ticks.len() {
            for &(note, velocity, start_tick) in &notes_on_for_ticks[tick] {
                let frequency = music::note_midi(440.0, note as usize);
                let loudness = (6.908 * (velocity as f64 / 255.0)).exp() / 1000.0;
                let attack = 0.01;
                let decay = 1.0;
                let start_t = start_tick as f64 * 60.0 / song.bpm as f64 / song.time_unit as f64;
                let relative_t = t - start_t;
                out += loudness * wave::SquareWave(frequency)(t) *
                    filter::envelope(relative_t, attack, decay)

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

    peak_normalize(&samples)
}

#[test]
fn it_peak_normalizes() {
    let input_negative = vec![-2.0f64, 1.0, -1.0];
    let output_negative = peak_normalize(&input_negative);
    assert_eq!(output_negative, vec![-1.0f64, 0.5, -0.5]);

    let input_positive = vec![2.0f64, 1.0, -1.0];
    let output_positive = peak_normalize(&input_positive);
    assert_eq!(output_positive, vec![1.0f64, 0.5, -0.5])
}

#[cfg(test)]
use std::i8;

#[cfg(test)]
use std::i16;

#[test]
fn it_quantizes() {
    assert_eq!(i8::MAX, quantize::<i8>(1.0));
    // assert_eq!(i8::MIN + 1, quantize::<i8>(-1.0)); // Bad quantization behaviour?
    assert_eq!(i16::MAX, quantize::<i16>(1.0));
    assert_eq!(0.0f32, quantize::<f32>(0.0));
    // assert_eq!(u8::MAX, quantize::<u8>(1.0)); // TODO: Make quantization work for unsigned types
}
