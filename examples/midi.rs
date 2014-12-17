#![feature(unboxed_closures)]

extern crate synthrs;

use synthrs::synthesizer::{ make_samples_from_midi, quantize_samples };
use synthrs::writer::write_wav;

fn main() {
    write_wav("out/octave.wav", 44100,
        quantize_samples::<i16>(
            make_samples_from_midi(44100, "examples/assets/octave.mid")
        )
    ).ok().expect("failed");

    write_wav("out/seikilos.wav", 44100,
        quantize_samples::<i16>(
            make_samples_from_midi(44100, "examples/assets/seikilos.mid")
        )
    ).ok().expect("failed");

    write_wav("out/danube.wav", 44100,
        quantize_samples::<i16>(
            make_samples_from_midi(44100, "examples/assets/danube.mid")
        )
    ).ok().expect("failed");

    write_wav("out/mountainking.wav", 44100,
        quantize_samples::<i16>(
            make_samples_from_midi(44100, "examples/assets/mountainking.mid")
        )
    ).ok().expect("failed");

    write_wav("out/rustle.wav", 44100,
        quantize_samples::<i16>(
            make_samples_from_midi(44100, "examples/assets/rustle.mid")
        )
    ).ok().expect("failed");
}
