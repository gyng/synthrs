#![feature(unboxed_closures)]

extern crate synthrs;

use synthrs::synthesizer::make_sample_16;
use synthrs::wave::SineWave;
use synthrs::writer::write_wav;

fn main() {
    // Telecomms
    write_wav("out/dialtone.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        0.5 * (SineWave(350.0)(t) + SineWave(440.0)(t))
    })).ok().expect("failed");

    write_wav("out/busysignal.wav", 44100, make_sample_16(8.0, 44100, |t: f64| -> f64 {
        if t % 1.0 < 0.5 {
            0.5 * (SineWave(480.0)(t) + SineWave(620.0)(t))
        } else {
            0.0
        }
    })).ok().expect("failed");

    write_wav("out/fastbusysignal.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        if t % 0.5 < 0.25 {
            0.5 * (SineWave(480.0)(t) + SineWave(620.0)(t))
        } else {
            0.0
        }
    })).ok().expect("failed");

    write_wav("out/offhook.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        if t % 0.2 < 0.1 {
            0.25 * (
                SineWave(1400.0)(t) + SineWave(2060.0)(t) +
                SineWave(2450.0)(t) + SineWave(2600.0)(t))
        } else {
            0.0
        }
    })).ok().expect("failed");

    write_wav("out/ring.wav", 44100, make_sample_16(15.0, 44100, |t: f64| -> f64 {
        if t % 6.0 < 2.0 {
            0.50 * (SineWave(440.0)(t) + SineWave(480.0)(t))
        } else {
            0.0
        }
    })).ok().expect("failed");
}
