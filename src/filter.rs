use std::f64::consts::PI;
use std::num::Float;
use std::num::FloatMath;


/// Cutoff: fraction of sample rate (eg. frequencies below sample_rate / cutoff are preserved)
/// Transition band: fraction of sample rate (how harsh a cutoff this is)
pub fn lowpass_filter(cutoff: f64, band: f64) -> Vec<f64> {
    let mut n = (4.0 / band).ceil() as uint;
    if n % 2 == 1 { n += 1; }

    let sinc = |x: f64| -> f64 {
        (x * PI).sin() / (x * PI)
    };

    let sinc_wave = Vec::from_fn(n, |i| {
        sinc(2.0 * cutoff * (i as f64 - (n as f64 - 1.0) / 2.0))
    });

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

pub fn blackman_window(size: uint) -> Vec<f64> {
    Vec::from_fn(size, |i| {
        0.42 - 0.5 * (2.0 * PI * i as f64 / (size as f64 - 1.0)).cos()
        + 0.08 * (4.0 * PI * i as f64 / (size as f64 - 1.0)).cos()
    })
}

pub fn highpass_filter(cutoff: f64, band: f64) -> Vec<f64> {
    spectral_invert(lowpass_filter(cutoff, band))
}

pub fn bandpass_filter(low_frequency: f64, high_frequency: f64, band: f64) -> Vec<f64> {
    assert!(low_frequency <= high_frequency);
    let lowpass = lowpass_filter(high_frequency, band);
    let highpass = highpass_filter(low_frequency, band);
    convolve(highpass, lowpass)
}

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
    let h_len = (filter.len() / 2) as int;

    for i in range(-(filter.len() as int / 2), input.len() as int - 1) {
        output.push(0.0);
        for j in range(0i, filter.len() as int) {
            let input_idx = i + j;
            let output_idx = i + h_len;
            if input_idx < 0 || input_idx >= input.len() as int { continue }
            output[output_idx as uint] += input[input_idx as uint] * filter[j as uint]
        }
    }

    output
}

pub fn add(left: Vec<f64>, right: Vec<f64>) -> Vec<f64> {
    left.iter().zip(right.iter()).map(|tup| {
        *tup.0 + *tup.1
    }).collect()
}

pub fn cutoff_from_frequency(frequency: f64, sample_rate: uint) -> f64 {
    frequency / sample_rate as f64
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
