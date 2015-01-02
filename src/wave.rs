use std::f64::consts::PI;
use std::num::Float;
use std::num::FloatMath;

#[deriving(Copy)]
pub struct SineWave(pub f64);

impl Fn<(f64, ), f64> for SineWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SineWave(frequency) = *self;
        FloatMath::sin(t * frequency * 2.0 * PI)
    }
}

#[deriving(Copy)]
pub struct SquareWave(pub f64);

impl Fn<(f64, ), f64> for SquareWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SquareWave(frequency) = *self;
        let sin_wave = SineWave(frequency);
        if sin_wave(t).is_positive() { 1.0 } else { -1.0 }
    }
}

#[deriving(Copy)]
pub struct SawtoothWave(pub f64);

impl Fn<(f64, ), f64> for SawtoothWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let SawtoothWave(frequency) = *self;
        let t_factor = t * frequency;
        t_factor - t_factor.floor() - 0.5
    }
}

#[deriving(Copy)]
pub struct TriangleWave(pub f64);

impl Fn<(f64, ), f64> for TriangleWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let TriangleWave(frequency) = *self;
        let sawtooth_wave = SawtoothWave(frequency);
        (sawtooth_wave(t).abs() - 0.25) * 4.0
    }
}

#[deriving(Copy)]
pub struct TangentWave(pub f64);

impl Fn<(f64, ), f64> for TangentWave {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let TangentWave(frequency) = *self;
        FloatMath::tan(t * frequency * PI).max(0.0).min(1.0)
    }
}

#[deriving(Copy)]
// http://computermusicresource.com/Simple.bell.tutorial.html
pub struct Bell(pub f64, pub f64, pub f64);

impl Fn<(f64, ), f64> for Bell {
    extern "rust-call" fn call(&self, (t, ): (f64, )) -> f64 {
        let Bell(frequency, attack, decay) = *self;

        let envelope = |t: f64, f: f64| -> f64 {
            let tf = t / f;
            if tf < attack {
                (tf / attack).min(1.0)
            } else {
                (1.0 - (tf / decay)).max(0.0)
            }
        };

        let h1 = SineWave(frequency * 0.56);
        let h2 = SineWave(frequency * 0.92);
        let h3 = SineWave(frequency * 1.19);
        let h4 = SineWave(frequency * 1.71);
        let h5 = SineWave(frequency * 2.00);
        let h6 = SineWave(frequency * 2.74);
        let h7 = SineWave(frequency * 3.00);
        let h8 = SineWave(frequency * 3.76);
        let h9 = SineWave(frequency * 4.07);

        (h1(t) * 1.0 * envelope(t, 1.0) +
         h2(t) * 0.5 * envelope(t, 2.0) +
         h3(t) * 0.25 * envelope(t, 4.0) +
         h4(t) * 0.125 * envelope(t, 6.0) +
         h5(t) * 0.0625 * envelope(t, 8.4) +
         h6(t) * 0.03125 * envelope(t, 10.8) +
         h7(t) * 0.015625 * envelope(t, 13.6) +
         h8(t) * 0.0078125 * envelope(t, 16.4) +
         h9(t) * 0.00390625 * envelope(t, 19.6)) / 2.0
    }
}
