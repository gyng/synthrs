# synthrs

Toy synthesiser in Rust.

## Usage

Edit `main.rs` to generate the sound you want then

    cargo run

or to run examples

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
