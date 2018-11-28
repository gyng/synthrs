//! A collection of waveform generating functions.
//!
//! Given a time `t` and `frequency`, returns the amplitude of the waveform
//! at the given time.
//!
//! `frequency` is in hertz (`44_100.0f64`)
//! `t` is in seconds; sample_rate is handled during synthesis
//!
//! Amplitude is in the range [-1, 1] and will be quantized or scaled to target bit depth
//!
//! See the short! source for each generator for exact details on what they do.

use std::collections::VecDeque;
use std::f64::consts::PI;

use crate::filter::envelope;

pub fn sine_wave(frequency: f64) -> impl Fn(f64) -> f64 {
    move |t| (t * frequency * 2.0 * PI).sin()
}

pub fn square_wave(frequency: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        let sin_wave = sine_wave(frequency);
        if sin_wave(t).is_sign_positive() {
            1.0
        } else {
            -1.0
        }
    }
}

pub fn sawtooth_wave(frequency: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        let t_factor = t * frequency;
        t_factor - t_factor.floor() - 0.5
    }
}

pub fn triangle_wave(frequency: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        let sawtooth_wave = sawtooth_wave(frequency);
        (sawtooth_wave(t).abs() - 0.25) * 4.0
    }
}

pub fn tangent_wave(frequency: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        (((t * frequency * PI) - 0.5).tan() / 4.0)
            .max(-1.0)
            .min(1.0)
    }
}

pub fn bell(frequency: f64, attack: f64, decay: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        // TODO: lazy-static this table
        // Frequency, amplitude, decay
        let harmonics_table: [(f64, f64, f64); 9] = [
            (0.56, 1.5, 1.0),
            (0.92, 0.5, 2.0),
            (1.19, 0.25, 4.0),
            (1.71, 0.125, 6.0),
            (2.00, 0.0625, 8.4),
            (2.74, 0.03125, 10.8),
            (3.00, 0.015625, 13.6),
            (3.76, 0.0078125, 16.4),
            (4.07, 0.00390625, 19.6),
        ];

        harmonics_table.iter().fold(0.0, |acc, h| {
            acc + sine_wave(frequency * h.0)(t) * h.1 * envelope(t, attack, decay * h.2)
        }) / 2.0
    }
}

pub fn organ(frequency: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        let frequency_2 = (frequency / 2.0) * 3.0;
        sine_wave(frequency)(t) + 0.2 * sine_wave(frequency_2)(t)
    }
}

/// Bastardised and butchered generic Karplus-Strong synthesis.
/// Try a Sawtooth, or even a Bell wave.
///
/// This is an example of a generator function using another generator function.
/// In this case, `karplus_strong` wraps around a generator function and
/// applies a poor emulation of a real-world object over it.
///
/// `attack` in seconds
/// `decay` in seconds
/// `sharpness` 0-1 is decent
/// `sample_rate` in hertz (eg, `44_100.0`)
///
/// ```
/// use synthrs::wave;
///
/// let karplus_sawtooth_generator =
///     |frequency: f64| {
///         wave::karplus_strong(wave::sawtooth_wave(frequency), 0.01, 1.0, 0.9, 44_100.0)
///     };
/// ```
pub fn karplus_strong<F: Fn(f64) -> f64>(
    generator: F,
    attack: f64,
    decay: f64,
    sharpness: f64,
    sample_rate: f64,
) -> impl Fn(f64) -> f64 {
    move |t| {
        let tick = 1.0 / sample_rate;

        // Instead of using delay_line_generator we manually unroll the loop here
        (0..10usize).fold(0.0, |acc, i| {
            acc + generator(t - tick * i as f64)
                * envelope(tick * i as f64, attack, decay)
                * sharpness.powf(i as f64)
        })
    }
}

pub fn noise() -> impl Fn(f64) -> f64 {
    |_t| rand::random::<f64>()
}

/// `sampler` creates a a generator function given a bunch of samples. Different frequencies are
/// generated using a simple pitch shift.
///
/// `frequency`: The frequency passed in to the generator
/// `sample`: The sample to be used. This is a `&'static Vec<f64>` and can be done during runtime with `lazy_static!`.
/// `sample_frequency`: The frequency of the sample provided. This is used to calculate how much to shift the pitch.
/// `sample_rate`: The sample rate of the given sample
/// ```compile_fail
/// use synthrs::sampler;
/// use synthrs::wave;
///
/// lazy_static! {
///    static ref SAMPLE: Vec<f64> =
///         sampler::samples_from_wave_file("test/assets/sine.wav").unwrap();
/// }
///
/// let frequency_to_generate = 110.0;
/// let sampler = wave::sampler(frequency_to_generate, &SAMPLE, 440.0, 44_100.0);
/// ```
pub fn sampler(
    frequency: f64,
    sample: &'static Vec<f64>,
    sample_frequency: f64,
    sample_rate: f64,
) -> impl Fn(f64) -> f64 {
    move |t| {
        let multiplier = frequency / sample_frequency;
        let original_index = sample_rate * t;
        let adjusted_index = (multiplier * original_index).round() as usize;

        if adjusted_index >= sample.len() {
            0.0
        } else {
            sample[adjusted_index]
        }
    }
}

/// Wwraps a generator function, delaying its output by `delay_length_samples` number of samples.
/// This isn't very useful in most cases because generator will likely change due to frequency changes.alloc
/// Look at `::crate::filter::DelayLine` for a more stateful filter that works on generated samples instead for most use cases.
///
/// `generator`: The generator to delay
/// `delay_length`: Seconds to delay by
/// `sample_rate`: The sample rate of the given sample
///
/// ```
/// use synthrs::sampler;
/// use synthrs::wave;
///
/// // This creates a sine wave that's delayed by 1 second
/// let generator = wave::sine_wave(440.0);
/// let delayed_sine = wave::delay_line_generator(generator, 1.0, 44_100);
/// ```
pub fn delay_line_generator<F: Fn(f64) -> f64>(
    generator: F,
    delay_length: f64,
    sample_rate: usize,
) -> impl Fn(f64) -> f64 {
    let delay_length_samples = (delay_length * sample_rate as f64).floor() as usize;
    let buf: VecDeque<f64> = VecDeque::with_capacity(delay_length_samples + 1);
    let cell = std::cell::RefCell::new(buf);

    move |t| {
        let mut buf = cell.borrow_mut();
        let current_sample = generator(t);

        let output = if buf.len() < delay_length_samples {
            0.0f64
        } else {
            buf.pop_front().unwrap_or(0.0f64)
        };

        buf.push_back(current_sample);
        output
    }
}

/// `rising_linear` is a stateful generator function.
/// Starting from `start_frequency`, it increases the output frequency by `increment_per_sample`
/// each time it is called, and loops back to `start_frequency` when it is above `end_frequency`.
///
/// This is mainly an example on how to do stateful generator functions.
/// This is achieved using interior mutability. See the source for details on how this is achieved.
pub fn rising_linear(
    start_frequency: f64,
    end_frequency: f64,
    increment_per_sample: f64,
) -> impl Fn(f64) -> f64 {
    // Our state! You can use a `RefCell` or a `Cell` for a start.
    // This example uses a `RefCell`, but a `Cell` will be simpler and
    // suffice for this simple state. An example using `Cell` is provided below.
    let cell = std::cell::RefCell::new(start_frequency);

    move |t| {
        let mut current_frequency = cell.borrow_mut();

        *current_frequency += increment_per_sample;

        if *current_frequency > end_frequency {
            *current_frequency = start_frequency;
        }

        sine_wave(*current_frequency)(t)
    }

    // The below is an example using `Cell` instead of `RefCell`.
    //
    // ```
    // let mut cell = std::cell::Cell::new(start_frequency);
    //
    // move |t| {
    //     let current_frequency = cell.get();
    //
    //     let new_frequency = if current_frequency > end_frequency {
    //         start_frequency
    //     } else {
    //         current_frequency + increment_per_sample
    //     };
    //
    //     cell.set(new_frequency);
    //
    //     sine_wave(new_frequency)(t)
    // }
    // ```
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_delay_line() {
        let identity = |t| t;
        let delayed = delay_line_generator(identity, 3.0, 1);

        assert_eq!(delayed(1.0), 0.0);
        assert_eq!(delayed(3.0), 0.0);
        assert_eq!(delayed(5.0), 0.0);
        assert_eq!(delayed(7.0), 1.0);
        assert_eq!(delayed(11.0), 3.0);
        assert_eq!(delayed(13.0), 5.0);
        assert_eq!(delayed(17.0), 7.0);
        assert_eq!(delayed(19.0), 11.0);
    }
}
