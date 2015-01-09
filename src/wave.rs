use std::f64::consts::PI;
use std::num::Float;

use filter::envelope;

#[derive(Copy)]
pub struct SineWave(pub f64);

impl Fn<(f64, ), f64> for SineWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SineWave(frequency) = *self;
        Float::sin(t * frequency * 2.0 * PI)
    }
}

#[derive(Copy)]
pub struct SquareWave(pub f64);

impl Fn<(f64, ), f64> for SquareWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SquareWave(frequency) = *self;
        let sin_wave = SineWave(frequency);
        if sin_wave(t).is_positive() { 1.0 } else { -1.0 }
    }
}

#[derive(Copy)]
pub struct SawtoothWave(pub f64);

impl Fn<(f64, ), f64> for SawtoothWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SawtoothWave(frequency) = *self;
        let t_factor = t * frequency;
        t_factor - t_factor.floor() - 0.5
    }
}

#[derive(Copy)]
pub struct TriangleWave(pub f64);

impl Fn<(f64, ), f64> for TriangleWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let TriangleWave(frequency) = *self;
        let sawtooth_wave = SawtoothWave(frequency);
        (sawtooth_wave(t).abs() - 0.25) * 4.0
    }
}

#[derive(Copy)]
pub struct TangentWave(pub f64);

impl Fn<(f64, ), f64> for TangentWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let TangentWave(frequency) = *self;
        ((Float::tan(t * frequency * PI) - 0.5) / 4.0).max(-1.0).min(1.0)
    }
}

#[derive(Copy)]
// http://computermusicresource.com/Simple.bell.tutorial.html
pub struct Bell(pub f64, pub f64, pub f64);

impl Fn<(f64, ), f64> for Bell {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let Bell(frequency, attack, decay) = *self;

        // Frequency, amplitude, decay
        let harmonics_table: [(f64, f64, f64); 9] = [
            (0.56, 1.5,        1.0),
            (0.92, 0.5,        2.0),
            (1.19, 0.25,       4.0),
            (1.71, 0.125,      6.0),
            (2.00, 0.0625,     8.4),
            (2.74, 0.03125,    10.8),
            (3.00, 0.015625,   13.6),
            (3.76, 0.0078125,  16.4),
            (4.07, 0.00390625, 19.6)
        ];

        harmonics_table.iter().fold(0.0, |acc, h| {
            acc + SineWave(frequency * h.0)(t) * h.1 * envelope(t, attack, decay * h.2)
        }) / 2.0
    }
}

/// Bastardised and butchered generic Karplus-Strong synthesis.
/// Try a Sawtooth, or even a Bell wave.
///
/// `attack` in seconds
/// `decay` in seconds
/// `sharpness` 0-1 is decent
#[derive(Copy)]
pub struct KarplusStrong<'a, F>(pub F, pub f64, pub f64, pub f64, pub f64);

impl<'a, F> Fn<(f64, ), f64> for KarplusStrong<'a, F> where F: Fn<(f64, ), f64> {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let KarplusStrong(ref wave, attack, decay, sharpness, sample_rate) = *self;

        let tick = 1.0 / sample_rate;

        // Pretend we have a delay feature in synthrs, manually unroll delay loops
        // Any given sample at any given time will have "imaginary past" loops in it
        range(0, 10us).fold(0.0, |acc, i| {
            acc + wave.call((t - tick * i as f64, ))
                * envelope(t + tick * i as f64, attack, decay)
                * sharpness.powf(i as f64)
        }) * envelope(t, attack, decay)
    }
}
