extern crate synthrs;

use synthrs::synthesizer::{make_samples, quantize_samples};
use synthrs::wave::sine_wave;
use synthrs::writer::write_wav_file;

fn main() {
    write_wav_file(
        "out/dialtone.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(15.0, 44_100, |t: f64| -> f64 {
            0.5 * (sine_wave(350.0)(t) + sine_wave(440.0)(t))
        })),
    )
    .expect("failed");

    write_wav_file(
        "out/busysignal.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(8.0, 44_100, |t: f64| -> f64 {
            if t % 1.0 < 0.5 {
                0.5 * (sine_wave(480.0)(t) + sine_wave(620.0)(t))
            } else {
                0.0
            }
        })),
    )
    .expect("failed");

    write_wav_file(
        "out/fastbusysignal.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(15.0, 44_100, |t: f64| -> f64 {
            if t % 0.5 < 0.25 {
                0.5 * (sine_wave(480.0)(t) + sine_wave(620.0)(t))
            } else {
                0.0
            }
        })),
    )
    .expect("failed");

    write_wav_file(
        "out/offhook.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(15.0, 44_100, |t: f64| -> f64 {
            if t % 0.2 < 0.1 {
                0.25 * (sine_wave(1400.0)(t)
                    + sine_wave(2060.0)(t)
                    + sine_wave(2450.0)(t)
                    + sine_wave(2600.0)(t))
            } else {
                0.0
            }
        })),
    )
    .expect("failed");

    write_wav_file(
        "out/ring.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples(15.0, 44_100, |t: f64| -> f64 {
            if t % 6.0 < 2.0 {
                0.50 * (sine_wave(440.0)(t) + sine_wave(480.0)(t))
            } else {
                0.0
            }
        })),
    )
    .expect("failed");
}
