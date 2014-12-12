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
