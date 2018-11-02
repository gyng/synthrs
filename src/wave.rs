//! A collection of waveform generating functions.
//!
//! Given a time `t` and `frequency`, returns the amplitude of the waveform
//! at the given time.

use std::f64::consts::PI;

use crate::filter::envelope;

#[derive(Clone, Copy)]
pub struct SineWave(pub f64);

impl Fn<(f64,)> for SineWave {
    extern "rust-call" fn call(&self, (t,): (f64,)) -> f64 {
        let SineWave(frequency) = *self;
        (t * frequency * 2.0 * PI).sin()
    }
}
impl FnMut<(f64,)> for SineWave {
    extern "rust-call" fn call_mut(&mut self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}
impl FnOnce<(f64,)> for SineWave {
    type Output = f64;
    extern "rust-call" fn call_once(self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}

#[derive(Clone, Copy)]
pub struct SquareWave(pub f64);

impl Fn<(f64,)> for SquareWave {
    extern "rust-call" fn call(&self, (t,): (f64,)) -> f64 {
        let SquareWave(frequency) = *self;
        let sin_wave = SineWave(frequency);
        if sin_wave(t).is_sign_positive() {
            1.0
        } else {
            -1.0
        }
    }
}
impl FnMut<(f64,)> for SquareWave {
    extern "rust-call" fn call_mut(&mut self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}
impl FnOnce<(f64,)> for SquareWave {
    type Output = f64;
    extern "rust-call" fn call_once(self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}

#[derive(Clone, Copy)]
pub struct SawtoothWave(pub f64);

impl Fn<(f64,)> for SawtoothWave {
    extern "rust-call" fn call(&self, (t,): (f64,)) -> f64 {
        let SawtoothWave(frequency) = *self;
        let t_factor = t * frequency;
        t_factor - t_factor.floor() - 0.5
    }
}
impl FnMut<(f64,)> for SawtoothWave {
    extern "rust-call" fn call_mut(&mut self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}
impl FnOnce<(f64,)> for SawtoothWave {
    type Output = f64;
    extern "rust-call" fn call_once(self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}

#[derive(Clone, Copy)]
pub struct TriangleWave(pub f64);

impl Fn<(f64,)> for TriangleWave {
    extern "rust-call" fn call(&self, (t,): (f64,)) -> f64 {
        let TriangleWave(frequency) = *self;
        let sawtooth_wave = SawtoothWave(frequency);
        (sawtooth_wave(t).abs() - 0.25) * 4.0
    }
}
impl FnMut<(f64,)> for TriangleWave {
    extern "rust-call" fn call_mut(&mut self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}
impl FnOnce<(f64,)> for TriangleWave {
    type Output = f64;
    extern "rust-call" fn call_once(self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}

#[derive(Clone, Copy)]
pub struct TangentWave(pub f64);

impl Fn<(f64,)> for TangentWave {
    extern "rust-call" fn call(&self, (t,): (f64,)) -> f64 {
        let TangentWave(frequency) = *self;
        (((t * frequency * PI) - 0.5).tan() / 4.0).max(-1.0).min(
            1.0,
        )
    }
}
impl FnMut<(f64,)> for TangentWave {
    extern "rust-call" fn call_mut(&mut self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}
impl FnOnce<(f64,)> for TangentWave {
    type Output = f64;
    extern "rust-call" fn call_once(self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}

#[derive(Clone, Copy)]
// http://computermusicresource.com/Simple.bell.tutorial.html
pub struct Bell(pub f64, pub f64, pub f64);

impl Fn<(f64,)> for Bell {
    extern "rust-call" fn call(&self, (t,): (f64,)) -> f64 {
        let Bell(frequency, attack, decay) = *self;

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
            acc + SineWave(frequency * h.0)(t) * h.1 * envelope(t, attack, decay * h.2)
        }) / 2.0
    }
}
impl FnMut<(f64,)> for Bell {
    extern "rust-call" fn call_mut(&mut self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}
impl FnOnce<(f64,)> for Bell {
    type Output = f64;
    extern "rust-call" fn call_once(self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}

/// Bastardised and butchered generic Karplus-Strong synthesis.
/// Try a Sawtooth, or even a Bell wave.
///
/// `attack` in seconds
/// `decay` in seconds
/// `sharpness` 0-1 is decent
#[derive(Clone, Copy)]
pub struct KarplusStrong<F>(pub F, pub f64, pub f64, pub f64, pub f64);

impl<F> Fn<(f64,)> for KarplusStrong<F>
where
    F: Fn(f64) -> f64,
{
    extern "rust-call" fn call(&self, (t,): (f64,)) -> f64 {
        let KarplusStrong(ref wave, attack, decay, sharpness, sample_rate) = *self;

        let tick = 1.0 / sample_rate;

        // Pretend we have a delay feature in synthrs, manually unroll delay loops
        // Any given sample at any given time will have "imaginary past" loops in it
        (0..10usize).fold(0.0, |acc, i| {
            acc +
                wave.call((t - tick * i as f64,)) * envelope(t + tick * i as f64, attack, decay) *
                    sharpness.powf(i as f64)
        }) * envelope(t, attack, decay)
    }
}
impl<F> FnMut<(f64,)> for KarplusStrong<F>
where
    F: Fn(f64) -> f64,
{
    extern "rust-call" fn call_mut(&mut self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}
impl<F> FnOnce<(f64,)> for KarplusStrong<F>
where
    F: Fn(f64) -> f64,
{
    type Output = f64;
    extern "rust-call" fn call_once(self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}

#[derive(Clone, Copy)]
pub struct Noise;

impl Fn<(f64,)> for Noise {
    extern "rust-call" fn call(&self, (_t,): (f64,)) -> f64 {
        rand::random::<f64>()
    }
}
impl FnMut<(f64,)> for Noise {
    extern "rust-call" fn call_mut(&mut self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}
impl FnOnce<(f64,)> for Noise {
    type Output = f64;
    extern "rust-call" fn call_once(self, (t,): (f64,)) -> f64 {
        self.call((t,))
    }
}
