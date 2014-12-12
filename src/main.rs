#![feature(lang_items, unboxed_closures)]

use std::io::{File, IoResult, Truncate, Write};
use std::f64::consts::PI;
use std::num::Float;
use std::num::FloatMath;
use std::rand;
use std::rand::Rng;

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
/// Transition band: fraction of sample rate
fn lowpass_filter(cutoff: f64, band: f64) -> Vec<f64> {
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
        println!("{} {} {}", acc, el, acc + *el);
        acc + *el
    });

    filter.iter().map(|el| {
        // println!("{}", *el / sum);
        *el / sum
        // *el
    }).collect()
}

fn convolve(filter: Vec<f64>, input: Vec<f64>) -> Vec<f64> {
    let mut output: Vec<f64> = input.clone();
    for i in range(filter.len() / 2, input.len() - filter.len() / 2) {
        let mut sum = 0.0;
        for j in range(0u, filter.len()) {
            if i + j > input.len() { continue; }
            sum += input[i + j - filter.len() / 2] * filter[j];
        }
        output[i] = sum / filter.len() as f64;
    }
    output
}

fn generate<F>(x: f64, f: F) -> f64 where F: Fn<(f64, ), f64> {
    f(x)
}

fn quantize_16(y: f64) -> i16 {
    // Quantization levels for 16 bits
    let levels = 2.0.powf(16.0) - 1.0;

    // Convert from [-1, 1] to [-2**16 / 2, 2**16 / 2]
    (y * (levels / 2.0)) as i16
}

fn quantize_sample_16(samples: Vec<f64>) -> Vec<i16> {
    samples.iter().map(|s| {
        quantize_16(*s)
    }).collect()
}

fn make_sample_16<F>(length: f64, sample_rate: uint, waveform: F) -> Vec<i16> where F: Fn<(f64, ), f64>+Copy {
    let num_samples = (sample_rate as f64 * length).floor() as uint;
    let mut samples: Vec<i16> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sample_rate as f64;
        samples.push(quantize_16(generate(t, waveform)));
    }

    samples
}

fn make_sample<F>(length: f64, sample_rate: uint, waveform: F) -> Vec<f64> where F: Fn<(f64, ), f64>+Copy {
    let num_samples = (sample_rate as f64 * length).floor() as uint;
    let mut samples: Vec<f64> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sample_rate as f64;
        samples.push(generate(t, waveform));
    }

    samples
}

fn write_pcm(filename: &str, samples: Vec<i16>) -> IoResult<()> {
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
fn write_wav(filename: &str, sample_rate: uint, samples: Vec<i16>) -> IoResult<()> {
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
    println!("Hello, synthrs!");

    write_pcm("out/sin.pcm", make_sample_16(1.0, 44100, SineWave(440.0))).ok().expect("Failed");
    write_wav("out/sin.wav", 44100, make_sample_16(1.0, 44100, SineWave(440.0))).ok().expect("Failed");
    write_wav("out/square.wav", 44100, make_sample_16(1.0, 44100, SquareWave(440.0))).ok().expect("Failed");
    write_wav("out/sawtooth.wav", 44100, make_sample_16(1.0, 44100, SawtoothWave(440.0))).ok().expect("Failed");

    write_wav("out/wolftone.wav", 44100, make_sample_16(1.0, 44100, |t: f64| -> f64 {
        (SquareWave(1000.0)(t) + SquareWave(1020.0)(t)) / 2.0
    })).ok().expect("Failed");

    write_wav("out/whitenoise.wav", 44100, make_sample_16(1.0, 44100, |_t: f64| -> f64 {
        let mut rng = rand::task_rng();
        (rng.gen::<f64>() - 0.5) * 2.0
    })).ok().expect("Failed");

    write_wav("out/rising.wav", 44100, make_sample_16(1.0, 44100, |t: f64| -> f64 {
        let (min_f, max_f) = (1000.0, 8000.0);
        let max_t = 1.0; // Duration of clip in seconds
        let range = max_f - min_f;
        let f = max_f - (max_t - t) * range;
        SineWave(f)(t)
    })).ok().expect("Failed");

    write_wav("out/racecar.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        let mut rng = rand::task_rng();
        let mut out = 0.0;
        if t < 14.0 { out += SawtoothWave(40.63 * (t / 2.0))(t); } // Engine
        if t < 1.0 { out += SawtoothWave(30.0)(t) * 10.0; } // Engine start
        if t < 14.0 && t.tan() > 0.0 { out += SquareWave(t.fract() * 130.8125)(t); } // Gear
        if t < 14.0 && t.tan() < 0.0 { out += SquareWave(t.sin() * 261.625)(t); } // Gear
        if t < 14.0 && t.tan() > 0.866 { out += SquareWave(t.fract() * 523.25)(t); } // Gear
        if t > 14.0 { out += (rng.gen::<f64>() - 0.5) * 8.0 } // Tree
        (out / 4.0).min(1.0)
    })).ok().expect("Failed");

    write_wav("out/shepard.wav", 44100, make_sample_16(30.0, 44100, |t: f64| -> f64 {
        let length = 10.0;
        let t_mod = t % length;
        let progress = t_mod / length;
        let tone_a2 = SineWave(110.0 / (1.0 + progress));
        let tone_a3 = SineWave(220.0 / (1.0 + progress));
        let tone_a4 = SineWave(440.0 / (1.0 + progress));
        let tone_a5 = SineWave(880.0 / (1.0 + progress));
        (tone_a2(t_mod) * (1.0 - progress)
        + tone_a5(t_mod) * progress
        + tone_a3(t_mod) + tone_a4(t_mod)) / 4.0
    })).ok().expect("Failed");

    // Telecomms
    write_wav("out/dialtone.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        0.5 * (SineWave(350.0)(t) + SineWave(440.0)(t))
    })).ok().expect("failed");

    write_wav("out/busysignal.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        if t % 1.0 < 0.5 {
            0.5 * (SineWave(480.0)(t) + SineWave(620.0)(t))
        } else {
            0.0
        }
    })).ok().expect("failed");

    write_wav("out/fastbusysignal.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        if t % 0.5 < 0.25 {
            0.5 * (SineWave(480.0)(t) + SineWave(620.0)(t))
        } else {
            0.0
        }
    })).ok().expect("failed");

    write_wav("out/offhook.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        if t % 0.2 < 0.1 {
            0.25 * (
                SineWave(1400.0)(t) + SineWave(2060.0)(t) +
                SineWave(2450.0)(t) + SineWave(2600.0)(t))
        } else {
            0.0
        }
    })).ok().expect("failed");

    write_wav("out/ring.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        if t % 6.0 < 2.0 {
            0.50 * (SineWave(440.0)(t) + SineWave(480.0)(t))
        } else {
            0.0
        }
    })).ok().expect("failed");

    // Lowpass filter/convolution example
    let filter = lowpass_filter(0.1, 0.08);
    let sample = make_sample(1.0, 44100, |t: f64| -> f64 {
        0.5 * (SineWave(6000.0)(t) + SineWave(1500.0)(t))
    });
    write_wav("out/lowpass.wav", 44100,
        quantize_sample_16(sample.clone()) + quantize_sample_16(convolve(filter, sample))
    ).ok().expect("Failed");
}


#[test]
fn it_convolves() {
    let filter = vec!(1.0, 1.0, 1.0);
    let input = vec!(0.0, 3.0, 0.0, 3.0, 0.0);
    let output = vec!(0.0, 1.0, 2.0, 1.0, 0.0);
    assert_eq!(convolve(filter, input), output);
}
