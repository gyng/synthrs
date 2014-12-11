# synthrs

Toy synthesiser in Rust.

## Usage

Edit `main.rs` to generate the sound you want then

    cargo run

This generates WAV or PCM files which can be opened in Audacity.

### Audacity PCM import settings

`File > Import > Raw Data...`

* Signed 16-bit PCM
* Little-endian
* 1 Channel (Mono)
* Sample rate: 44100Hz

## License

synthrs is licensed under the MIT License. See `LICENSE` for details.
