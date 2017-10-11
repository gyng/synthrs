# synthrs

[![Build Status](https://travis-ci.org/gyng/synthrs.svg?branch=master)](https://travis-ci.org/gyng/synthrs)

[Documentation](https://gyng.github.io/synthrs)

Toy synthesiser library in Rust. Requires Rust nightly for now (ie. does not compile on stable).

## Features

* Not too difficult syntax for writing your own tones (see examples)
* Basic filters (low-pass, high-pass, band-pass, band-reject, attack/decay envelope)
* Basic waveforms (sine, square, triangle, sawtooth, tangent, bastardised Karplus-Strong)
* MIDI synthesis
* PCM or WAV output

#### Examples (loud)

* [usage with cpal](https://github.com/gyng/midcat)
* [busy signal](examples/assets/busysignal.ogg)
* [bell](examples/assets/bell.ogg)
* [mtnking-pure](examples/assets/mountainking-puresquare.ogg) *pure square wave*
* [mtnking-envelope](examples/assets/mountainking.ogg) *butchered Karplus-Strong square wave with attack and decay*
* [rustle](examples/assets/rustle.ogg)
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
use synthrs::wave::SineWave;
use synthrs::writer::write_wav;

fn main() {
    write_wav("out/sin.wav", 44_100,
        &quantize_samples::<i16>(
            &make_samples(1.0, 44_100, SineWave(440.0))
        )
    ).expect("failed to write to file");

}
```

More examples are in [`examples/`](examples/).

### Examples

Check out [`Cargo.toml`](Cargo.toml) for the full example list.

* [`simple`](examples/simple.rs) generates simple tones in `out/`
* [`telecoms`](examples/telecoms.rs) generates phone tones in `out/`
* [`filters`](examples/filters.rs) generates examples of audio filtering in `out/`
* [`midi`](examples/midi.rs) synthesises a few MIDI files in `out/`

This generates WAV or PCM files which can be opened in Audacity. Example MIDI files are public domain as far as I can tell.

#### cpal

[Example usage with cpal](https://github.com/gyng/midcat)

### Audacity PCM import settings

`File > Import > Raw Data...`

* Signed 16-bit PCM
* Little-endian
* 1 Channel (Mono)
* Sample rate: 44_100Hz

## License

synthrs is licensed under the MIT License. See [`LICENSE`](LICENSE) for details.
