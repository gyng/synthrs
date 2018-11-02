#![feature(unboxed_closures)]

extern crate synthrs;

use synthrs::synthesizer::{make_samples, peak_normalize, quantize_samples, SamplesIter};
use synthrs::wave::{
    bell, karplus_strong, noise, rising_linear, sawtooth_wave, sine_wave, square_wave,
    tangent_wave, triangle_wave,
};
use synthrs::writer::{write_pcm, write_wav};

fn main() {
    // This creates a sine wave for 1.0s at 44_100Hz
    // 0. `sine_wave` create a 440Hz sine function
    // 1. `make_samples` creates a `Vec` of samples from the `sine_wave` function
    //    from 0.0 to 1.0 seconds at a 44_100Hz sample rate
    // 2. `quantize_samples::<i16>` quantizes the floating-point samples as a signed 16-bit int
    // 3. `write_pcm` writes the samples to a PCM file
    write_pcm(
        "out/sine.pcm",
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, sine_wave(440.0))),
    ).expect("failed");

    // Write to a WAV file
    write_wav(
        "out/sine.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, sine_wave(440.0))),
    ).expect("failed");

    // `make_samples` takes in an Fn closure of type `|t: f64| -> f64`, where `t` = seconds
    write_wav(
        "out/sine_closure.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, |t| {
            (t * 440.0 * 2.0 * 3.14159).sin()
        })),
    ).expect("failed to write to file");

    // `quantize_samples` takes in an interator, which the `make_samples` function returns
    let sine_iter = SamplesIter::new(44_100, Box::new(sine_wave(440.0)));
    write_wav(
        "out/sine_iter.wav",
        44_100,
        &quantize_samples::<i16>(sine_iter.take(44_100).collect::<Vec<f64>>().as_slice()),
    ).expect("failed");

    write_wav(
        "out/square.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, square_wave(440.0))),
    ).expect("failed");

    write_wav(
        "out/sawtooth.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, sawtooth_wave(440.0))),
    ).expect("failed");

    write_wav(
        "out/triangle.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, triangle_wave(440.0))),
    ).expect("failed");

    write_wav(
        "out/tangent.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, tangent_wave(440.0))),
    ).expect("failed");

    write_wav(
        "out/noise.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, noise())),
    ).expect("failed");

    // Custom function for tone generation, t is in seconds
    write_wav(
        "out/wolftone.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, |t: f64| -> f64 {
            (square_wave(1000.0)(t) + square_wave(1020.0)(t)) / 2.0
        })),
    ).expect("failed");

    write_wav(
        "out/rising.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(1.0, 44_100, |t: f64| -> f64 {
            let (min_f, max_f) = (1000.0, 8000.0);
            let max_t = 1.0; // Duration of clip in seconds
            let range = max_f - min_f;
            let f = max_f - (max_t - t) * range;
            sine_wave(f)(t)
        })),
    ).expect("failed");

    write_wav(
        "out/rising_wub.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(
            3.0,
            44_100,
            rising_linear(440.0, 1760.0, 0.1),
        )),
    ).expect("failed");

    write_wav(
        "out/bell.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(10.0, 44_100, |t: f64| -> f64 {
            bell(200.0, 0.003, 0.5)(t)
        })),
    ).expect("failed");

    write_wav(
        "out/karplus_strong.wav",
        44_100,
        &quantize_samples::<i16>(&peak_normalize(&make_samples(
            5.0,
            44_100,
            |t: f64| -> f64 { karplus_strong(sawtooth_wave(440.0), 0.01, 1.0, 0.9, 44_100.0)(t) },
        ))),
    ).expect("failed");

    write_wav(
        "out/racecar.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(15.0, 44_100, |t: f64| -> f64 {
            let mut out = 0.0;
            if t < 14.0 {
                out += sawtooth_wave(40.63 * (t / 2.0))(t);
            } // Engine
            if t < 1.0 {
                out += sawtooth_wave(30.0)(t) * 10.0;
            } // Engine start
            if t < 14.0 && t.tan() > 0.0 {
                out += square_wave(t.fract() * 130.8125)(t);
            } // Gear
            if t < 14.0 && t.tan() < 0.0 {
                out += square_wave(t.sin() * 261.625)(t);
            } // Gear
            if t < 14.0 && t.tan() > 0.866 {
                out += square_wave(t.fract() * 523.25)(t);
            } // Gear
            if t > 14.0 {
                out += tangent_wave(100.0)(t)
            } // Tree
            (out / 4.0).min(1.0)
        })),
    ).expect("failed");

    write_wav(
        "out/shepard.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(30.0, 44_100, |t: f64| -> f64 {
            let length = 10.0;
            let t_mod = t % length;
            let progress = t_mod / length;
            let tone_a2 = sine_wave(110.0 / (1.0 + progress));
            let tone_a3 = sine_wave(220.0 / (1.0 + progress));
            let tone_a4 = sine_wave(440.0 / (1.0 + progress));
            let tone_a5 = sine_wave(880.0 / (1.0 + progress));
            (tone_a2(t_mod) * (1.0 - progress)
                + tone_a5(t_mod) * progress
                + tone_a3(t_mod)
                + tone_a4(t_mod))
                / 4.0
        })),
    ).expect("failed");
}
