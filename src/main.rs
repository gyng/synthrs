#![feature(lang_items, unboxed_closures)]
#![allow(dead_code)]

use std::io::{File, IoResult, Truncate, Write};
use std::f64::consts::PI;
use std::num::Float;
use std::num::FloatMath;

struct SineWave(pub f64);

impl Fn<(f64, ), f64> for SineWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SineWave(frequency) = *self;
        FloatMath::sin(t * frequency * 2.0 * PI)
    }
}

struct SquareWave(pub f64);

impl Fn<(f64, ), f64> for SquareWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SquareWave(frequency) = *self;
        let sin_wave = SineWave(frequency);
        if sin_wave(t).is_positive() { 1.0 } else { -1.0 }
    }
}

struct SawtoothWave(pub f64);

impl Fn<(f64, ), f64> for SawtoothWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SawtoothWave(frequency) = *self;
        let t_factor = t * frequency;
        t_factor - t_factor.floor()
    }
}

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

    let blackman_window = Vec::from_fn(n, |i| {
        0.42 - 0.5 * (2.0 * PI * i as f64 / (n as f64 - 1.0)).cos()
        + 0.08 * (4.0 * PI * i as f64 / (n as f64 - 1.0)).cos()
    });

    let filter: Vec<f64> =  sinc_wave.iter().zip(blackman_window.iter()).map(|tup| {
        *tup.val0() * *tup.val1()
    }).collect();

    // Normalize
    let sum = filter.iter().fold(0.0, |acc, el| {
        acc + *el
    });

    filter.iter().map(|el| {
        *el / sum
    }).collect()
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

    filter.iter().map(|el| {
        let add = if count == filter.len() / 2 { 1.0 } else { 0.0 };
        count += 1;
        -*el + add
    }).collect()
}

// Output will be longer than input as we add to border
pub fn convolve(filter: Vec<f64>, input: Vec<f64>) -> Vec<f64> {
    let mut output: Vec<f64> = Vec::new();
    let h_len = (filter.len() / 2) as int;

    for i in range(-(filter.len() as int / 2), input.len() as int - 1) {
        output.push(0.0);
        for j in range(0i, filter.len() as int) {
            let input_idx = i + j;
            let output_idx = i + h_len;
            if input_idx < 0 || input_idx >= input.len() as int { continue; }
            output[output_idx as uint] += input[input_idx as uint] * filter[j as uint]
        }
    }

    output
}

pub fn add(left: Vec<f64>, right: Vec<f64>) -> Vec<f64> {
    left.iter().zip(right.iter()).map(|tup| {
        *tup.val0() + *tup.val1()
    }).collect()
}

pub fn cutoff_from_frequency(frequency: f64, sample_rate: uint) -> f64 {
    frequency / sample_rate as f64
}

pub fn generate<F>(x: f64, f: F) -> f64 where F: Fn<(f64, ), f64> {
    f(x)
}

pub fn quantize_16(y: f64) -> i16 {
    // Quantization levels for 16 bits
    let levels = 2.0.powf(16.0) - 1.0;

    // Convert from [-1, 1] to [-2**16 / 2, 2**16 / 2]
    (y * (levels / 2.0)) as i16
}

pub fn quantize_sample_16(samples: Vec<f64>) -> Vec<i16> {
    samples.iter().map(|s| {
        quantize_16(*s)
    }).collect()
}

pub fn make_sample_16<F>(length: f64, sample_rate: uint, waveform: F) -> Vec<i16> where F: Fn<(f64, ), f64>+Copy {
    let num_samples = (sample_rate as f64 * length).floor() as uint;
    let mut samples: Vec<i16> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sample_rate as f64;
        samples.push(quantize_16(generate(t, waveform)));
    }

    samples
}

pub fn make_sample<F>(length: f64, sample_rate: uint, waveform: F) -> Vec<f64> where F: Fn<(f64, ), f64>+Copy {
    let num_samples = (sample_rate as f64 * length).floor() as uint;
    let mut samples: Vec<f64> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sample_rate as f64;
        samples.push(generate(t, waveform));
    }

    samples
}

/// Equal-tempered
/// 0=C 1=C# 2=D 3=D# 4=E 5=F 6=F# 7=G 8=G# 9=A 10=B
pub fn note(a4: f64, note: uint, harmonic: uint) -> f64 {
    let semitones_from_a4 = harmonic as int * 12 + note as int - 9 - 48;
    a4 * (semitones_from_a4 as f64 * 2.0.ln() / 12.0).exp()
}

pub fn write_pcm(filename: &str, samples: Vec<i16>) -> IoResult<()> {
    let path = Path::new(filename);
    let mut f = match File::open_mode(&path, Truncate, Write) {
        Ok(f) => f,
        Err(e) => panic!("File error: {}", e)
    };

    for sample in samples.iter() {
        try!(f.write_le_i16(*sample));
    }

    Ok(())
}

// See: https://ccrma.stanford.edu/courses/422/projects/WaveFormat/
pub fn write_wav(filename: &str, sample_rate: uint, samples: Vec<i16>) -> IoResult<()> {
    let path = Path::new(filename);
    let mut f = match File::open_mode(&path, Truncate, Write) {
        Ok(f) => f,
        Err(e) => panic!("File error: {}", e)
    };

    // Some WAV header fields
    let channels = 1;
    let bit_depth = 16;
    let subchunk_2_size = samples.len() * channels * bit_depth / 8;
    let chunk_size = 36 + subchunk_2_size as i32;
    let byte_rate = (sample_rate * channels * bit_depth / 8) as i32;
    let block_align = (channels * bit_depth / 8) as i16;

    try!(f.write_be_i32(0x52494646))              // ChunkID, RIFF
    try!(f.write_le_i32(chunk_size));             // ChunkSize
    try!(f.write_be_i32(0x57415645));             // Format, WAVE

    try!(f.write_be_i32(0x666d7420));             // Subchunk1ID, fmt
    try!(f.write_le_i32(16));                     // Subchunk1Size, 16 for PCM
    try!(f.write_le_i16(1));                      // AudioFormat, PCM = 1 (linear quantization)
    try!(f.write_le_i16(channels as i16));        // NumChannels
    try!(f.write_le_i32(sample_rate as i32));     // SampleRate
    try!(f.write_le_i32(byte_rate));              // ByteRate
    try!(f.write_le_i16(block_align));            // BlockAlign
    try!(f.write_le_i16(bit_depth as i16));       // BitsPerSample

    try!(f.write_be_i32(0x64617461));             // Subchunk2ID, data
    try!(f.write_le_i32(subchunk_2_size as i32)); // Subchunk2Size, number of bytes in the data

    for sample in samples.iter() {
        try!(f.write_le_i16(*sample))
    }

    Ok(())
}

fn main() {
    println!("Hello, synthrs!\n Use cargo run --example simple to try. Files are generated in /out. Check Cargo.toml for a list of examples.");
}


#[test]
fn it_convolves() {
    let filter = vec!(1.0, 1.0, 1.0);
    let input = vec!(0.0, 0.0, 3.0, 0.0, 3.0, 0.0, 0.0);
    let output = vec!(0.0, 3.0, 3.0, 6.0, 3.0, 3.0, 0.0);
    assert_eq!(convolve(filter, input), output);
}

#[test]
fn it_equal_tempers() {
    let threshold = 0.1;
    let c4 = 261.63;
    let a4 = 440.0;
    let d3 = 146.83;
    let fs6 = 1479.98;
    assert!(FloatMath::abs_sub(note(a4, 9, 4), a4) < threshold);
    assert!(FloatMath::abs_sub(note(a4, 0, 4), c4) < threshold);
    assert!(FloatMath::abs_sub(note(a4, 2, 3), d3) < threshold);
    assert!(FloatMath::abs_sub(note(a4, 6, 6), fs6) < threshold);
}
