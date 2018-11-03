#![feature(unboxed_closures)]

extern crate synthrs;

use synthrs::midi;
use synthrs::synthesizer::{make_samples_from_midi, make_samples_from_midi_file, quantize_samples};
use synthrs::wave;
use synthrs::writer::write_wav;

fn main() {
    // `make_samples_from_midi_file` is a convenience function that parses and synthesises
    // a MIDI file given a file path
    // Set `use_envelope` to decide whether to use a basic attack/decay envelope when generating samples
    // The envelope will slowly fade each note out over time
    write_wav(
        "out/octave.wav",
        44_100,
        &quantize_samples::<i16>(
            &make_samples_from_midi_file(
                wave::square_wave,
                44_100,
                true,
                "examples/assets/octave.mid",
            ).unwrap(),
        ),
    ).expect("failed");

    // Pass in any generator to `make_samples_from_midi_file`!
    write_wav(
        "out/octave_bell.wav",
        44_100,
        &quantize_samples::<i16>(
            &make_samples_from_midi_file(
                |frequency: f64| wave::bell(frequency, 0.003, 0.5),
                44_100,
                false,
                "examples/assets/octave.mid",
            ).unwrap(),
        ),
    ).expect("failed");

    // Manually parse and synthesise MIDI files
    // The `make_samples_from_midi` function works on an already-parsed MIDI file
    // `read_midi` does the file reading and parsing
    let song = midi::read_midi("examples/assets/octave.mid").unwrap();
    write_wav(
        "out/octave_no_envelope.wav",
        44_100,
        &quantize_samples::<i16>(
            &make_samples_from_midi(wave::square_wave, 44_100, false, song).unwrap(),
        ),
    ).expect("failed");

    // Seikilos: the oldest known surviving musical composition
    // https://en.wikipedia.org/wiki/Seikilos_epitaph
    write_wav(
        "out/seikilos.wav",
        44_100,
        &quantize_samples::<i16>(
            &make_samples_from_midi_file(
                |frequency: f64| {
                    wave::karplus_strong(wave::sawtooth_wave(frequency), 0.01, 1.0, 0.9, 44_100.0)
                },
                44_100,
                true,
                "examples/assets/seikilos.mid",
            ).unwrap(),
        ),
    ).expect("failed");

    // Johann Strauss II - The Blue Danube
    write_wav(
        "out/danube.wav",
        44_100,
        &quantize_samples::<i16>(
            &make_samples_from_midi_file(
                wave::sine_wave,
                44_100,
                true,
                "examples/assets/danube.mid",
            ).unwrap(),
        ),
    ).expect("failed");

    // Grieg - In the Hall of the Mountain King
    write_wav(
        "out/mountainking.wav",
        44_100,
        &quantize_samples::<i16>(
            &make_samples_from_midi_file(
                wave::square_wave,
                44_100,
                true,
                "examples/assets/mountainking.mid",
            ).unwrap(),
        ),
    ).expect("failed");

    // Christian Sinding - Rustle of Spring (Frühlingsrauschen)
    write_wav(
        "out/rustle.wav",
        44_100,
        &quantize_samples::<i16>(
            &make_samples_from_midi_file(
                wave::square_wave,
                44_100,
                true,
                "examples/assets/rustle.mid",
            ).unwrap(),
        ),
    ).expect("failed");
}
