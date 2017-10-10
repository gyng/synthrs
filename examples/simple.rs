#![feature(unboxed_closures)]

extern crate synthrs;

use synthrs::synthesizer::{make_samples, quantize_samples, peak_normalize, SamplesIter};
use synthrs::wave::{SineWave, SquareWave, SawtoothWave, TriangleWave, TangentWave, Bell,
                    KarplusStrong, Noise};
use synthrs::writer::{write_pcm, write_wav};

fn main() {
    // This creates a sine wave for 1.0s at 44100Hz
    // 0. `SineWave` create a 440Hz sine function
    // 1. `make_samples` creates a `Vec` of samples from the `SineWave` function
    //    from 0.0 to 1.0 seconds at a 44100Hz sample rate
    // 2. `quantize_samples::<i16>` quantizes the floating-point samples as a signed 16-bit int
    // 3. `write_pcm` writes the samples to a PCM file
    write_pcm(
        "out/sin.pcm",
        &quantize_samples::<i16>(&make_samples(1.0, 44100, SineWave(440.0))),
    ).expect("failed");

    write_wav(
        "out/sin.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(1.0, 44100, SineWave(440.0))),
    ).expect("failed");

    let sine_iter = SamplesIter::new(44100, Box::new(SineWave(440.0)));
    write_wav(
        "out/sin_iter.wav",
        44100,
        &quantize_samples::<i16>(sine_iter.take(44100).collect::<Vec<f64>>().as_slice()),
    ).expect("failed");

    write_wav(
        "out/square.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(1.0, 44100, SquareWave(440.0))),
    ).expect("failed");

    write_wav(
        "out/sawtooth.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(1.0, 44100, SawtoothWave(440.0))),
    ).expect("failed");

    write_wav(
        "out/triangle.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(1.0, 44100, TriangleWave(440.0))),
    ).expect("failed");

    write_wav(
        "out/tangent.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(1.0, 44100, TangentWave(440.0))),
    ).expect("failed");

    write_wav(
        "out/noise.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(1.0, 44100, Noise)),
    ).expect("failed");

    // Custom function for tone generation, t is in seconds
    write_wav(
        "out/wolftone.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(1.0, 44100, |t: f64| -> f64 {
            (SquareWave(1000.0)(t) + SquareWave(1020.0)(t)) / 2.0
        })),
    ).expect("failed");

    write_wav(
        "out/rising.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(1.0, 44100, |t: f64| -> f64 {
            let (min_f, max_f) = (1000.0, 8000.0);
            let max_t = 1.0; // Duration of clip in seconds
            let range = max_f - min_f;
            let f = max_f - (max_t - t) * range;
            SineWave(f)(t)
        })),
    ).expect("failed");

    write_wav(
        "out/bell.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(
            10.0,
            44100,
            |t: f64| -> f64 { Bell(200.0, 0.003, 0.5)(t) },
        )),
    ).expect("failed");

    write_wav(
        "out/karplusstrong.wav",
        44100,
        &quantize_samples::<i16>(&peak_normalize(&make_samples(5.0, 44100, |t: f64| -> f64 {
            KarplusStrong(SawtoothWave(440.0), 0.01, 1.0, 0.9, 44100.0)(t)
        }))),
    ).expect("failed");

    write_wav(
        "out/racecar.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(15.0, 44100, |t: f64| -> f64 {
            let mut out = 0.0;
            if t < 14.0 {
                out += SawtoothWave(40.63 * (t / 2.0))(t);
            } // Engine
            if t < 1.0 {
                out += SawtoothWave(30.0)(t) * 10.0;
            } // Engine start
            if t < 14.0 && t.tan() > 0.0 {
                out += SquareWave(t.fract() * 130.8125)(t);
            } // Gear
            if t < 14.0 && t.tan() < 0.0 {
                out += SquareWave(t.sin() * 261.625)(t);
            } // Gear
            if t < 14.0 && t.tan() > 0.866 {
                out += SquareWave(t.fract() * 523.25)(t);
            } // Gear
            if t > 14.0 {
                out += TangentWave(100.0)(t)
            } // Tree
            (out / 4.0).min(1.0)
        })),
    ).expect("failed");

    write_wav(
        "out/shepard.wav",
        44100,
        &quantize_samples::<i16>(&make_samples(30.0, 44100, |t: f64| -> f64 {
            let length = 10.0;
            let t_mod = t % length;
            let progress = t_mod / length;
            let tone_a2 = SineWave(110.0 / (1.0 + progress));
            let tone_a3 = SineWave(220.0 / (1.0 + progress));
            let tone_a4 = SineWave(440.0 / (1.0 + progress));
            let tone_a5 = SineWave(880.0 / (1.0 + progress));
            (tone_a2(t_mod) * (1.0 - progress) + tone_a5(t_mod) * progress + tone_a3(t_mod) +
                 tone_a4(t_mod)) / 4.0
        })),
    ).expect("failed");
}
