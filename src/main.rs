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


fn generate<F>(x: f64, f: F) -> f64 where F: Fn<(f64, ), f64> {
    f(x)
}

fn quantize_16(y: f64) -> i16 {
    // Quantization levels for 16 bits
    let levels = 2.0.powf(16.0) - 1.0;

    // Convert from [-1, 1] to [-2**16 / 2, 2**16 / 2]
    (y * (levels / 2.0)) as i16
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

    write_pcm("out/sin.pcm", make_sample_16(1.0, 44100, SineWave(440.0))).ok();
    write_wav("out/sin.wav", 44100, make_sample_16(1.0, 44100, SineWave(440.0))).ok();
    write_wav("out/square.wav", 44100, make_sample_16(1.0, 44100, SquareWave(440.0))).ok();
    write_wav("out/sawtooth.wav", 44100, make_sample_16(1.0, 44100, SawtoothWave(440.0))).ok();

    write_wav("out/wolftone.wav", 44100, make_sample_16(1.0, 44100, |t: f64| -> f64 {
        (SquareWave(1000.0)(t) + SquareWave(1020.0)(t)) / 2.0
    })).ok();

    write_wav("out/whitenoise.wav", 44100, make_sample_16(1.0, 44100, |_t: f64| -> f64 {
        let mut rng = rand::task_rng();
        (rng.gen::<f64>() - 0.5) * 2.0
    })).ok();

    write_wav("out/rising.wav", 44100, make_sample_16(1.0, 44100, |t: f64| -> f64 {
        let (min_f, max_f) = (1000.0, 8000.0);
        let max_t = 1.0; // Duration of clip in seconds
        let range = max_f - min_f;
        let f = max_f - (max_t - t) * range;
        SineWave(f)(t)
    })).ok();

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
    })).ok();

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
    })).ok();
}
