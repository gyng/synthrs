//! A collection of signal filters.
//!
//! To filter a bunch of samples, first create the filter and samples.
//!
//! There are two types of filters: stateless, and stateful filters.
//! Stateless filters can be used to convolve samples, while stateful filters transform individual samples.
//!
//! ### Stateless filters
//!
//! Stateless filters are pure functions and are used in conjunction with the convolve function:
//! ```
//! use synthrs::filter::{convolve, cutoff_from_frequency, lowpass_filter};
//! use synthrs::synthesizer::{quantize_samples, make_samples};
//! use synthrs::wave::sine_wave;
//!
//! // Generate a bunch of samples at two different frequencies
//! let samples = make_samples(0.5, 44_100, |t: f64| -> f64 {
//!     0.5 * (sine_wave(6000.0)(t) + sine_wave(80.0)(t))
//! });
//!
//! // Create a lowpass filter, using a cutoff of 400Hz at a 44_100Hz sample rate (ie. filter out frequencies >400Hz)
//! let lowpass = lowpass_filter(cutoff_from_frequency(400.0, 44_100), 0.01);
//!
//! // Apply convolution to filter out high frequencies
//! let lowpass_samples = quantize_samples::<i16>(&convolve(&lowpass, &samples));
//! ```
//! #### Common stateless filter arguments:
//!
//! * `cutoff`: as a fraction of sample rate, can be obtained from
//!             `cutoff_from_frequency(cutoff, sample_rate)`. (eg. for a lowpass filter
//!             frequencies below `sample_rate` / `cutoff` are preserved)
//! * `band`: transition band as a fraction of the sample rate. This determines how
//!         the cutoff "blends", or how harsh a cutoff this is.
//!
//! ### Stateful filters
//!
//! Stateful filters are structs which hold some state, such as `DelayLine` which has to
//! keep in memory historical samples.
//!
//! They can be used to transform a bunch of samples using `map`.
//!
//! ```
//! use synthrs::filter::Comb;
//! use synthrs::synthesizer::{quantize_samples, make_samples};
//! use synthrs::wave::sine_wave;
//!
//! // Creates a comb filter with
//! // * 0.2 second delay
//! // * 44100Hz,
//! // * 0.5 dampening inverse factor
//! // * 0.5 dampening factor
//! // * 0.5 feedback factor
//! let mut comb = Comb::new(0.2, 44_100, 0.5, 0.5, 0.5);
//!
//! let samples = make_samples(0.5, 44_100, |t: f64| -> f64 { sine_wave(440.0)(t) });
//!
//! let filtered_raw: Vec<f64> = samples
//!     .into_iter()
//!     .map(|sample| comb.tick(sample))
//!     .collect();
//! let filtered_quantized = quantize_samples::<i16>(&filtered_raw);
//! ```
//!
//! See: `examples/filters.rs`
//!
//! An all-poss filter is implemented as a generator in `crate::wave::allpass`.

use std::f64::consts::PI;

/// Creates a low-pass filter. Frequencies below the cutoff are preserved when
/// samples are convolved with this filter.
pub fn lowpass_filter(cutoff: f64, band: f64) -> Vec<f64> {
    let mut n = (4.0 / band).ceil() as usize;
    if n % 2 == 1 {
        n += 1;
    }

    let sinc = |x: f64| -> f64 { (x * PI).sin() / (x * PI) };

    let sinc_wave: Vec<f64> = (0..n)
        .map(|i| sinc(2.0 * cutoff * (i as f64 - (n as f64 - 1.0) / 2.0)))
        .collect();

    let blackman_window = blackman_window(n);

    let filter: Vec<f64> = sinc_wave
        .iter()
        .zip(blackman_window.iter())
        .map(|tup| *tup.0 * *tup.1)
        .collect();

    // Normalize
    let sum = filter.iter().fold(0.0, |acc, &el| acc + el);

    filter.iter().map(|&el| el / sum).collect()
}

/// Creates a Blackman window filter of a given size.
pub fn blackman_window(size: usize) -> Vec<f64> {
    (0..size)
        .map(|i| {
            0.42 - 0.5 * (2.0 * PI * i as f64 / (size as f64 - 1.0)).cos()
                + 0.08 * (4.0 * PI * i as f64 / (size as f64 - 1.0)).cos()
        })
        .collect()
}

/// Creates a high-pass filter. Frequencies above the cutoff are preserved when
/// samples are convolved with this filter.
pub fn highpass_filter(cutoff: f64, band: f64) -> Vec<f64> {
    spectral_invert(&lowpass_filter(cutoff, band))
}

/// Creates a low-pass filter. Frequencies between `low_frequency` and `high_frequency`
/// are preserved when samples are convolved with this filter.
pub fn bandpass_filter(low_frequency: f64, high_frequency: f64, band: f64) -> Vec<f64> {
    assert!(low_frequency <= high_frequency);
    let lowpass = lowpass_filter(high_frequency, band);
    let highpass = highpass_filter(low_frequency, band);
    convolve(&highpass, &lowpass)
}

/// Creates a low-pass filter. Frequencies outside of `low_frequency` and `high_frequency`
/// are preserved when samples are convolved with this filter.
pub fn bandreject_filter(low_frequency: f64, high_frequency: f64, band: f64) -> Vec<f64> {
    assert!(low_frequency <= high_frequency);
    let lowpass = lowpass_filter(low_frequency, band);
    let highpass = highpass_filter(high_frequency, band);
    add(&highpass, &lowpass)
}

/// Given a filter, inverts it. For example, inverting a low-pass filter will result in a
/// high-pass filter with the same cutoff frequency.
pub fn spectral_invert(filter: &[f64]) -> Vec<f64> {
    assert_eq!(filter.len() % 2, 0);
    let mut count = 0;

    filter
        .iter()
        .map(|&el| {
            let add = if count == filter.len() / 2 { 1.0 } else { 0.0 };
            count += 1;
            -el + add
        })
        .collect()
}

pub fn convolve(filter: &[f64], input: &[f64]) -> Vec<f64> {
    let mut output: Vec<f64> = Vec::new();
    let h_len = (filter.len() / 2) as isize;

    for i in -(filter.len() as isize / 2)..(input.len() as isize - 1) {
        output.push(0.0);
        for j in 0isize..filter.len() as isize {
            let input_idx = i + j;
            let output_idx = i + h_len;
            if input_idx < 0 || input_idx >= input.len() as isize {
                continue;
            }
            output[output_idx as usize] += input[input_idx as usize] * filter[j as usize]
        }
    }

    output
}

/// Performs elementwise addition of two `Vec<f64>`s. Can be used to combine filters together
/// (eg. combining a low-pass filter with a high-pass filter to create a band-pass filter)
pub fn add(left: &[f64], right: &[f64]) -> Vec<f64> {
    left.iter()
        .zip(right.iter())
        .map(|tup| *tup.0 + *tup.1)
        .collect()
}

/// Returns the cutoff fraction for a given cutoff frequency at a sample rate, which can be
/// used for filter creation.
pub fn cutoff_from_frequency(frequency: f64, sample_rate: usize) -> f64 {
    frequency / sample_rate as f64
}

/// Simple linear attack/decay envelope. No sustain or release.
pub fn envelope(relative_t: f64, attack: f64, decay: f64) -> f64 {
    if relative_t < 0.0 {
        return 0.0;
    } else if relative_t < attack {
        return relative_t / attack;
    } else if relative_t < attack + decay {
        return 1.0 - (relative_t - attack) / decay;
    }

    0.0
}

/// A stateful delay line. Samples are delayed for `delay_length` seconds.
///
/// https://en.wikipedia.org/wiki/Analog_delay_line
///
/// ```
/// use synthrs::filter::AllPass;
///
/// let mut allpass = AllPass::new(1.0, 44_100, 0.5);
/// let samples: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
///
/// let filtered = samples.into_iter().map(|sample| allpass.tick(sample));
/// ```
///
/// Taken from: https://github.com/irh/freeverb-rs/blob/master/freeverb/src/delay_line.rs
#[derive(Clone, Debug)]
pub struct DelayLine {
    pub buf: Vec<f64>,
    index: usize,
    pub delay_length: f64,
    pub delay_samples: usize,
    pub sample_rate: usize,
}

impl DelayLine {
    /// Creates a new delay line. Samples are delayed for `delay_length` seconds.
    pub fn new(delay_length: f64, sample_rate: usize) -> DelayLine {
        let delay_samples = ((delay_length * sample_rate as f64).round() + 1.0) as usize;

        DelayLine {
            buf: vec![0.0; delay_samples],
            index: 0,
            delay_length,
            delay_samples,
            sample_rate,
        }
    }

    pub fn read(&self) -> f64 {
        self.buf[self.index]
    }

    pub fn write(&mut self, value: f64) {
        self.buf[self.index] = value;

        if self.index == self.buf.len() - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }
}

/// A stateful all-pass filter.
///
/// https://en.wikipedia.org/wiki/All-pass_filter
///
/// ```
/// use synthrs::filter::AllPass;
///
/// let mut allpass = AllPass::new(1.0, 44_100, 0.5);
/// let samples: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
///
/// let filtered = samples.into_iter().map(|sample| allpass.tick(sample));
/// ```
///
/// Taken from: https://github.com/irh/freeverb-rs/blob/master/freeverb/src/all_pass.rs
#[derive(Clone, Debug)]
pub struct AllPass {
    delay_line: DelayLine,
    /// Feedback multiplier (0.5 works)
    pub feedback: f64,
}

impl AllPass {
    /// Creates a new all-pass filter. Samples are delayed for `delay_length` seconds.
    pub fn new(delay_length: f64, sample_rate: usize, feedback: f64) -> AllPass {
        AllPass {
            delay_line: DelayLine::new(delay_length, sample_rate),
            feedback,
        }
    }

    pub fn tick(&mut self, input: f64) -> f64 {
        let delayed = self.delay_line.read();
        self.delay_line.write(input + delayed * self.feedback);
        -input + delayed
    }
}

/// A stateful comb filter.
///
/// https://en.wikipedia.org/wiki/Comb_filter
///
/// ```
/// use synthrs::filter::Comb;
///
/// let mut comb = Comb::new(1.0, 44_100, 0.5, 0.5, 0.5);
/// let samples: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
///
/// let filtered = samples.into_iter().map(|sample| comb.tick(sample));
/// ```
///
/// Taken from: https://github.com/irh/freeverb-rs/blob/master/freeverb/src/comb.rs
#[derive(Clone, Debug)]
pub struct Comb {
    delay_line: DelayLine,
    filter_state: f64,
    /// 0.5 works
    pub dampening_inverse: f64,
    /// 0.5 works
    pub dampening: f64,
    /// 0.5 works
    pub feedback: f64,
}

impl Comb {
    /// Creates a new comb filter. Samples are delayed for `delay_length` seconds.
    pub fn new(
        delay_length: f64,
        sample_rate: usize,
        dampening_inverse: f64,
        dampening: f64,
        feedback: f64,
    ) -> Comb {
        Comb {
            dampening_inverse,
            dampening,
            delay_line: DelayLine::new(delay_length, sample_rate),
            feedback,
            filter_state: 0.0,
        }
    }

    pub fn tick(&mut self, input: f64) -> f64 {
        let output = self.delay_line.read();
        self.filter_state = output * self.dampening_inverse + self.filter_state * self.dampening;
        self.delay_line
            .write(input + self.filter_state * self.feedback);

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convolve() {
        let filter = vec![1.0, 1.0, 1.0];
        let input = vec![0.0, 0.0, 3.0, 0.0, 3.0, 0.0, 0.0];
        let output = vec![0.0, 3.0, 3.0, 6.0, 3.0, 3.0, 0.0];
        assert_eq!(convolve(&filter, &input), output);
    }

    #[test]
    fn test_add() {
        let a = vec![1.0, -1.0, -8.0];
        let b = vec![-1.0, 5.0, 3.0];
        let expected = vec![0.0, 4.0, -5.0];
        assert_eq!(add(&a, &b), expected);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_envelope() {
        assert_eq!(envelope(0.25, 1.0, 1.0), 0.25);
        assert_eq!(envelope(0.5, 1.0, 1.0), 0.5);
        assert_eq!(envelope(1.0, 1.0, 1.0), 1.0);
        assert_eq!(envelope(1.5, 1.0, 1.0), 0.5);
        assert_eq!(envelope(3.0, 1.0, 1.0), 0.0);
        assert_eq!(envelope(-0.5, 1.0, 1.0), 0.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_delay_line() {
        let mut delay_line = DelayLine::new(3.0, 1);

        delay_line.write(1.0);
        assert_eq!(delay_line.read(), 0.0);
        delay_line.write(3.0);
        assert_eq!(delay_line.read(), 0.0);
        delay_line.write(5.0);
        assert_eq!(delay_line.read(), 0.0);
        delay_line.write(7.0);
        assert_eq!(delay_line.read(), 1.0);
        delay_line.write(11.0);
        assert_eq!(delay_line.read(), 3.0);
        delay_line.write(13.0);
        assert_eq!(delay_line.read(), 5.0);
        delay_line.write(17.0);
        assert_eq!(delay_line.read(), 7.0);
    }
}
