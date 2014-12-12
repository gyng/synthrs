#![feature(unboxed_closures)]

extern crate synthrs;

use std::rand;
use std::rand::Rng;
use std::num::Float;
use std::num::FloatMath;

use synthrs::synthesizer::{ make_sample, quantize_sample_16 };
use synthrs::wave::{ SineWave, SquareWave, SawtoothWave };
use synthrs::writer::{ write_pcm, write_wav };

fn main() {
    // This creates a sine wave for 1.0s at 44100Hz
    // 0. SineWave create a 440Hz sine function
    // 1. make_sample creates a Vec of samples from the SineWave function
    //    from 0.0 to 1.0 seconds at a 44100Hz sample rate
    // 2. quantize_sample_16 quantizes the floating-point samples at 16 bits
    // 3. write_pcm writes the samples to a PCM file
    write_pcm("out/sin.pcm",
        quantize_sample_16(make_sample(1.0, 44100, SineWave(440.0)))
    ).ok().expect("failed");

    write_wav("out/sin.wav", 44100,
        quantize_sample_16(make_sample(1.0, 44100, SineWave(440.0)))
    ).ok().expect("failed");

    write_wav("out/square.wav", 44100,
        quantize_sample_16(make_sample(1.0, 44100, SquareWave(440.0)))
    ).ok().expect("failed");

    write_wav("out/sawtooth.wav", 44100,
        quantize_sample_16(make_sample(1.0, 44100, SawtoothWave(440.0)))
    ).ok().expect("failed");

    // Custom function for tone generation, t is in seconds
    write_wav("out/wolftone.wav", 44100,
        quantize_sample_16(
            make_sample(1.0, 44100, |t: f64| -> f64 {
                (SquareWave(1000.0)(t) + SquareWave(1020.0)(t)) / 2.0
            })
        )
    ).ok().expect("failed");

    write_wav("out/whitenoise.wav", 44100,
        quantize_sample_16(
            make_sample(1.0, 44100, |_t: f64| -> f64 {
                let mut rng = rand::task_rng();
                (rng.gen::<f64>() - 0.5) * 2.0
            })
        )
    ).ok().expect("failed");

    write_wav("out/rising.wav", 44100,
        quantize_sample_16(
            make_sample(1.0, 44100, |t: f64| -> f64 {
                let (min_f, max_f) = (1000.0, 8000.0);
                let max_t = 1.0; // Duration of clip in seconds
                let range = max_f - min_f;
                let f = max_f - (max_t - t) * range;
                SineWave(f)(t)
            })
        )
    ).ok().expect("failed");

    write_wav("out/racecar.wav", 44100,
        quantize_sample_16(
            make_sample(15.0, 44100, |t: f64| -> f64 {
                let mut rng = rand::task_rng();
                let mut out = 0.0;
                if t < 14.0 { out += SawtoothWave(40.63 * (t / 2.0))(t); } // Engine
                if t < 1.0 { out += SawtoothWave(30.0)(t) * 10.0; } // Engine start
                if t < 14.0 && t.tan() > 0.0 { out += SquareWave(t.fract() * 130.8125)(t); } // Gear
                if t < 14.0 && t.tan() < 0.0 { out += SquareWave(t.sin() * 261.625)(t); } // Gear
                if t < 14.0 && t.tan() > 0.866 { out += SquareWave(t.fract() * 523.25)(t); } // Gear
                if t > 14.0 { out += (rng.gen::<f64>() - 0.5) * 8.0 } // Tree
                (out / 4.0).min(1.0)
            })
        )
    ).ok().expect("failed");

    write_wav("out/shepard.wav", 44100,
        quantize_sample_16(
            make_sample(30.0, 44100, |t: f64| -> f64 {
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
            })
        )
    ).ok().expect("failed");
}
