# synthrs

[![Build Status](https://travis-ci.org/gyng/synthrs.svg?branch=master)](https://travis-ci.org/gyng/synthrs)

[Documentation](https://gyng.github.io/synthrs)

Toy synthesiser library in Rust. Requires Rust nightly for now (ie. does not compile on stable).

## Features

* Not too difficult syntax for writing your own tones (see examples)
* Basic filters (low-pass, high-pass, band-pass, band-reject, all-pass, comb, delay line, attack/decay envelope)
* Basic waveforms (sine, square, triangle, sawtooth, tangent, bastardised Karplus-Strong, and more)
* MIDI synthesis
* Basic sample synthesis (WAV)
* PCM or WAV output

#### Integrations

* [usage with cpal](https://github.com/gyng/midcat)
* [midi-to-wav](https://github.com/gyng/midi-to-wav)
* [wasm, web synthesis](https://gyng.github.io/synthrs-wasm-ts/#/)

#### Examples (loud)
* [busy signal](examples/assets/busysignal.ogg)
* [bell](examples/assets/bell.ogg)
* [mtnking-pure](examples/assets/mountainking-puresquare.ogg) *pure square wave*
* [mtnking-envelope](examples/assets/mountainking.ogg) *butchered Karplus-Strong square wave with attack and decay*
* [rustle](examples/assets/rustle.ogg)
* [clarinet sample](examples/assets/octave_clarinet_sampler.ogg)
* [assorted synthesized MIDIs](http://sugoi.pw/samples/) *pure square waves*

## Try

To run examples (list below)

    cargo run --example EXAMPLE_NAME

To use as a library, add this to `Cargo.toml`

    [dependencies.synthrs]
    git = "https://github.com/gyng/synthrs"
    rev = "the commit you want"

To write a custom tone to a WAV file

```rust
extern crate synthrs;

use synthrs::synthesizer::{ make_samples, quantize_samples };
use synthrs::wave::sine_wave;
use synthrs::writer::write_wav_file;

fn main() {
    // Using a predefined generator
    write_wav_file("out/sine.wav", 44_100,
        &quantize_samples::<i16>(
            &make_samples(1.0, 44_100, sine_wave(440.0))
        )
    ).expect("failed to write to file");

    // `make_samples` takes in the duration, sample rate, and a generator closure.
    // It returns an iterator which `quantize_samples` wraps around (setting the bit depth).
    write_wav_file("out/sine_closure.wav", 44_100,
        &quantize_samples::<i16>(
            &make_samples(1.0, 44_100, |t| (t * 440.0 * 2.0 * 3.14159).sin())
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

#### midi-to-wav

[midi-to-wav, a simple cli to convert MIDI files to WAV](https://github.com/gyng/midi-to-wav)

#### wasm

* [synthrs-wasm-ts](https://github.com/gyng/synthrs-wasm-ts)
* [demo](https://gyng.github.io/synthrs-wasm-ts)

### Audacity PCM import settings

`File > Import > Raw Data...`

* Signed 16-bit PCM
* Little-endian
* 1 Channel (Mono)
* Sample rate: 44_100Hz (or whatever your samples generated have)

## License

synthrs is licensed under the MIT License. See [`LICENSE`](LICENSE) for details.
