# synthrs

[![Build Status](https://travis-ci.org/gyng/synthrs.svg)](https://travis-ci.org/gyng/synthrs)

Toy synthesiser library in Rust. [(sample)](https://dl.dropboxusercontent.com/u/38256631/busysignal.ogg)

## Features

* Not too difficult syntax for writing your own tones (see examples)
* Basic filters (low-pass, high-pass, band-pass, band-reject)
* PCM or WAV output

## Try

To run examples

    cargo run --example EXAMPLE_NAME

### Examples

Check out `Cargo.toml` for the full example list.

* `simple` generates simple tones in `/out`
* `telecoms` generates phone tones in `/out`
* `filters` generates examples of audio filtering in `/out`

This generates WAV or PCM files which can be opened in Audacity.

### Audacity PCM import settings

`File > Import > Raw Data...`

* Signed 16-bit PCM
* Little-endian
* 1 Channel (Mono)
* Sample rate: 44100Hz

## License

synthrs is licensed under the MIT License. See `LICENSE` for details.
