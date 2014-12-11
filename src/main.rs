#![feature(lang_items, unboxed_closures)]

use std::io::{File, Truncate, Write};
use std::mem;
use std::num::Float;
use std::num::FloatMath;

struct SinWave(pub f64);

impl Fn<(f64, ), f64> for SinWave {
    extern "rust-call" fn call(&self, (x, ): (f64, )) -> f64 {
        let SinWave(period) = *self;
        FloatMath::sin(x * period)
    }
}

struct SquareWave(pub f64);

impl Fn<(f64, ), f64> for SquareWave {
    extern "rust-call" fn call(&self, (x, ): (f64, )) -> f64 {
        let SquareWave(period) = *self;
        let sin_wave = SinWave(period);
        if sin_wave(x).is_positive() { 1.0 } else { -1.0 }
    }
}

fn main() {
    println!("Hello, synthrs!");

    let mut bytes: [u8, ..44100] = unsafe { mem::uninitialized() };
    for i in range(0u, 44100) {
        // bytes[i] = quantize_8bit(generate(i as f64, SinWave(1000.0)))
        bytes[i] = quantize_8bit(generate(i as f64, SquareWave(1000.0)))
        // bytes[i] = generate(i as f64, |x: f64| {
        //    FloatMath::sin(x / 8000.0)
        // });
    }

    write_pcm("test.pcm", &bytes);
}

fn generate<F>(x: f64, f: F) -> f64 where F: Fn<(f64, ), f64> {
    f(x)
}

fn quantize_8bit(y: f64) -> u8 {
    let step = 2.0 / 8.0; // Max range [-1, 1], 8 bits
    (((y + 1.0) / step) * 8.0).floor() as u8 // Convert to [0, 7]
}

fn write_pcm(filename: &str, bytes: &[u8]) {
    let path = Path::new(filename);
    let mut f = match File::open_mode(&path, Truncate, Write) {
        Ok(f) => f,
        Err(e) => panic!("File error: {}", e)
    };

    f.write(bytes);
}
