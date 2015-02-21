//! A collection of signal filters. To filter a bunch of samples, first create the
//! filter and samples. Following that, run `convolve(filter, input)`.
//!
//! See: `examples/filters.rs`
//!
//! Common filter arguments:
//!
//! * `cutoff`: as a fraction of sample rate, can be obtained from
//!             `cutoff_from_frequency(cutoff, sample_rate)`. (eg. for a lowpass filter
//!             frequencies below sample_rate / cutoff are preserved)
//! * `band`: transition band as a fraction of the sample rate. This determines how
//!         the cutoff "blends", or how harsh a cutoff this is.

use std::f64::consts::PI;
use std::iter::range;
use std::num::Float;

/// Creates a low-pass filter. Frequencies below the cutoff are preserved when
/// samples are convolved with this filter.
pub fn lowpass_filter(cutoff: f64, band: f64) -> Vec<f64> {
    let mut n = (4.0 / band).ceil() as usize;
    if n % 2 == 1 { n += 1; }

    let sinc = |&: x: f64| -> f64 {
        (x * PI).sin() / (x * PI)
    };

    let sinc_wave: Vec<f64> = range(0, n).map(|i| {
        sinc(2.0 * cutoff * (i as f64 - (n as f64 - 1.0) / 2.0))
    }).collect();

    let blackman_window = blackman_window(n);

    let filter: Vec<f64> =  sinc_wave.iter().zip(blackman_window.iter()).map(|tup| {
        *tup.0 * *tup.1
    }).collect();

    // Normalize
    let sum = filter.iter().fold(0.0, |acc, &el| {
        acc + el
    });

    filter.iter().map(|&el| {
        el / sum
    }).collect()
}

pub fn blackman_window(size: usize) -> Vec<f64> {
    range(0, size).map(|i| {
        0.42 - 0.5 * (2.0 * PI * i as f64 / (size as f64 - 1.0)).cos()
        + 0.08 * (4.0 * PI * i as f64 / (size as f64 - 1.0)).cos()
    }).collect()
}

/// Creates a high-pass filter. Frequencies above the cutoff are preserved when
/// samples are convolved with this filter.
pub fn highpass_filter(cutoff: f64, band: f64) -> Vec<f64> {
    spectral_invert(lowpass_filter(cutoff, band))
}

/// Creates a low-pass filter. Frequencies between `low_frequency` and `high_frequency`
/// are preserved when samples are convolved with this filter.
pub fn bandpass_filter(low_frequency: f64, high_frequency: f64, band: f64) -> Vec<f64> {
    assert!(low_frequency <= high_frequency);
    let lowpass = lowpass_filter(high_frequency, band);
    let highpass = highpass_filter(low_frequency, band);
    convolve(highpass, lowpass)
}

/// Creates a low-pass filter. Frequencies outside of `low_frequency` and `high_frequency`
/// are preserved when samples are convolved with this filter.
pub fn bandreject_filter(low_frequency: f64, high_frequency: f64, band: f64) -> Vec<f64> {
    assert!(low_frequency <= high_frequency);
    let lowpass = lowpass_filter(low_frequency, band);
    let highpass = highpass_filter(high_frequency, band);
    add(highpass, lowpass)
}

pub fn spectral_invert(filter: Vec<f64>) -> Vec<f64> {
    assert_eq!(filter.len() % 2, 0);
    let mut count = 0;

    filter.iter().map(|&el| {
        let add = if count == filter.len() / 2 { 1.0 } else { 0.0 };
        count += 1;
        -el + add
    }).collect()
}

pub fn convolve(filter: Vec<f64>, input: Vec<f64>) -> Vec<f64> {
    let mut output: Vec<f64> = Vec::new();
    let h_len = (filter.len() / 2) as isize;

    for i in range(-(filter.len() as isize / 2), input.len() as isize - 1) {
        output.push(0.0);
        for j in range(0isize, filter.len() as isize) {
            let input_idx = i + j;
            let output_idx = i + h_len;
            if input_idx < 0 || input_idx >= input.len() as isize { continue }
            output[output_idx as usize] += input[input_idx as usize] * filter[j as usize]
        }
    }

    output
}

pub fn add(left: Vec<f64>, right: Vec<f64>) -> Vec<f64> {
    left.iter().zip(right.iter()).map(|tup| {
        *tup.0 + *tup.1
    }).collect()
}

/// Returns the cutoff fraction for a given cutoff frequency at a sample rate, which can be
/// used for filter creation.
pub fn cutoff_from_frequency(frequency: f64, sample_rate: usize) -> f64 {
    frequency / sample_rate as f64
}

/// Simple linear attack/decay envelope. No sustain or release.
pub fn envelope(relative_t: f64, attack: f64, decay: f64) -> f64 {
    if relative_t < 0.0 {
        0.0
    } else if relative_t < attack {
        relative_t / attack
    } else if relative_t < attack + decay {
        1.0 - (relative_t - attack) / decay
    } else {
        0.0
    }
}

#[test]
fn it_convolves() {
    let filter = vec!(1.0, 1.0, 1.0);
    let input = vec!(0.0, 0.0, 3.0, 0.0, 3.0, 0.0, 0.0);
    let output = vec!(0.0, 3.0, 3.0, 6.0, 3.0, 3.0, 0.0);
    assert_eq!(convolve(filter, input), output);
}

#[test]
fn it_does_elementwise_addition_of_two_samples() {
    let a = vec!(1.0, -1.0, -8.0);
    let b = vec!(-1.0, 5.0, 3.0);
    let expected = vec!(0.0, 4.0, -5.0);
    assert_eq!(add(a, b), expected);
}

#[test]
fn it_envelopes_a_value() {
    assert_eq!(envelope(0.25, 1.0, 1.0), 0.25);
    assert_eq!(envelope(0.5, 1.0, 1.0), 0.5);
    assert_eq!(envelope(1.0, 1.0, 1.0), 1.0);
    assert_eq!(envelope(1.5, 1.0, 1.0), 0.5);
    assert_eq!(envelope(3.0, 1.0, 1.0), 0.0);
    assert_eq!(envelope(-0.5, 1.0, 1.0), 0.0);
}
