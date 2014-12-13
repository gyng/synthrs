# synthrs

[![Build Status](https://travis-ci.org/gyng/synthrs.svg)](https://travis-ci.org/gyng/synthrs)

Toy synthesiser library in Rust.

Examples (loud)
[[busy signal]](https://dl.dropboxusercontent.com/u/38256631/busysignal.ogg)
[[bell]](https://dl.dropboxusercontent.com/u/38256631/bell.ogg)
[[rustle]](https://dl.dropboxusercontent.com/u/38256631/rustle.ogg)

## Features

* Not too difficult syntax for writing your own tones (see examples)
* Basic filters (low-pass, high-pass, band-pass, band-reject)
* MIDI format0 input
* PCM or WAV output

## Try

To run examples

    cargo run --example EXAMPLE_NAME

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
