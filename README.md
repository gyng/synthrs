# synthrs

[![Build Status](https://travis-ci.org/gyng/synthrs.svg)](https://travis-ci.org/gyng/synthrs)

Toy synthesiser library in Rust.

## Features

* Not too difficult syntax for writing your own tones (see examples)
* Basic filters (low-pass, high-pass, band-pass, band-reject, attack/decay envelope)
* Basic waveforms (sine, square, triangle, sawtooth, tangent, basterdised Karplus-Strong)
* MIDI synthesis
* PCM or WAV output

#### Examples (loud)

* [busy signal](https://dl.dropboxusercontent.com/u/38256631/busysignal.ogg)
* [bell](https://dl.dropboxusercontent.com/u/38256631/bell.ogg)
* [mtnking-pure](https://dl.dropboxusercontent.com/u/38256631/mountainking-puresquare.ogg) *pure square wave*
* [mtnking-envelope](https://dl.dropboxusercontent.com/u/38256631/mountainking.ogg) *butchered Karplus-Strong square wave with attack and decay*
* [rustle](https://dl.dropboxusercontent.com/u/38256631/rustle.ogg)
* [assorted synthesized MIDIs](http://sugoi.pw/samples/) *pure square waves*

## Try

To run examples (list below)

    cargo run --example EXAMPLE_NAME

To use as a library, add this to `Cargo.toml`

    [dependencies.synthrs]
    git = "https://github.com/gyng/synthrs"

To write a custom tone to a WAV file

```rust
extern crate synthrs;

use synthrs::synthesizer::{ make_samples, quantize_samples };
use synthrs::wave::SquareWave;
use synthrs::writer::write_wav;

fn main() {
    write_wav("out/wolftone.wav", 44100,
        quantize_samples::<i16>(
            make_samples(1.0, 44100, |t: f64| -> f64 {
                (SquareWave(1000.0)(t) + SquareWave(1020.0)(t)) / 2.0
            })
        )
    ).ok().expect("failed");
}
```

More examples are in `/example`.

### Examples

Check out `Cargo.toml` for the full example list.

* `simple` generates simple tones in `/out`
* `telecoms` generates phone tones in `/out`
* `filters` generates examples of audio filtering in `/out`
* `midi` synthesises a few MIDI files in `/out`

This generates WAV or PCM files which can be opened in Audacity. Example MIDI files are public domain as far as I can tell.

### Audacity PCM import settings

`File > Import > Raw Data...`

* Signed 16-bit PCM
* Little-endian
* 1 Channel (Mono)
* Sample rate: 44100Hz

## License

synthrs is licensed under the MIT License. See `LICENSE` for details.
