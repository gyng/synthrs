#![feature(lang_items, unboxed_closures)]

use std::io::{File, Truncate, Write};
use std::num::Float;
use std::num::FloatMath;
use std::rand;
use std::rand::Rng;

struct SinWave(pub f64);

impl Fn<(f64, ), f64> for SinWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SinWave(frequency) = *self;
        FloatMath::sin(t * frequency)
    }
}

struct SquareWave(pub f64);

impl Fn<(f64, ), f64> for SquareWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SquareWave(frequency) = *self;
        let sin_wave = SinWave(frequency);
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

fn make_sample_16<F>(length: f64, sampling_frequency: uint, waveform: F) -> Vec<i16> where F: Fn<(f64, ), f64>+Copy {
    let num_samples = (sampling_frequency as f64 * length).floor() as uint;
    let mut samples: Vec<i16> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sampling_frequency as f64;
        samples.push(quantize_16(generate(t, waveform)));
    }

    samples
}

fn write_pcm(filename: &str, samples: Vec<i16>) {
    let path = Path::new(filename);
    let mut f = match File::open_mode(&path, Truncate, Write) {
        Ok(f) => f,
        Err(e) => panic!("File error: {}", e)
    };

    for sample in samples.iter() {
        if f.write_le_i16(*sample).is_err() {
            panic!("Error writing file {}", filename);
        }
    }
}

fn main() {
    println!("Hello, synthrs!");
    write_pcm("sin.pcm", make_sample_16(1.0, 44100, SinWave(1000.0)));
    write_pcm("square.pcm", make_sample_16(1.0, 44100, SquareWave(1000.0)));
    write_pcm("sawtooth.pcm", make_sample_16(1.0, 44100, SawtoothWave(1000.0)));
    write_pcm("noise.pcm", make_sample_16(1.0, 44100, |_t: f64| -> f64 {
        let mut rng = rand::task_rng();
        (rng.gen::<f64>() - 0.5) * 2.0
    }));
    write_pcm("wolftone.pcm", make_sample_16(1.0, 44100, |t: f64| -> f64 {
        SinWave(1000.0)(t) + SinWave(1010.0)(t)
    }));
}
