#![feature(unboxed_closures)]

extern crate synthrs;

use synthrs::synthesizer::{make_samples_from_midi, quantize_samples};
use synthrs::writer::write_wav;

fn main() {
    write_wav(
        "out/octave.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples_from_midi(44_100, "examples/assets/octave.mid")
            .unwrap()),
    ).expect("failed");

    write_wav(
        "out/seikilos.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples_from_midi(
            44_100,
            "examples/assets/seikilos.mid",
        ).unwrap()),
    ).expect("failed");

    write_wav(
        "out/danube.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples_from_midi(44_100, "examples/assets/danube.mid")
            .unwrap()),
    ).expect("failed");

    write_wav(
        "out/mountainking.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples_from_midi(
            44_100,
            "examples/assets/mountainking.mid",
        ).unwrap()),
    ).expect("failed");

    write_wav(
        "out/rustle.wav",
        44_100,
        &quantize_samples::<i16>(&make_samples_from_midi(44_100, "examples/assets/rustle.mid")
            .unwrap()),
    ).expect("failed");
}
