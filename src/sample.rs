//! Functions for dealing with creating samples for sample-synthesis generators

use std::io::{Cursor, Result};

use crate::synthesizer::unquantize_samples;
use crate::writer::{read_wav, read_wav_file, Wave};

/// Given a `crate::writer::Wave`, extract a `Vec<f64>` of samples from it and the size of that vec
///
/// ```
/// use synthrs::sample::samples_from_wave;
/// use synthrs::writer::read_wav_file;
///
/// let wave = read_wav_file("./tests/assets/sine.wav").unwrap();
/// let (samples, num_samples) = samples_from_wave(wave);
/// ```
pub fn samples_from_wave(wave: Wave) -> (Vec<f64>, usize) {
    let samples = unquantize_samples(&wave.pcm);
    let length = samples.len();
    (samples, length)
}

/// Given a bunch of bytes for a wave file, extract a `Vec<f64>` of samples from it and the size of that vec
///
/// ```
/// use std::fs::OpenOptions;
/// use std::io::{BufReader, Read};
/// use std::path::Path;
/// use synthrs::sample::samples_from_wave_bytes;
/// use synthrs::writer::read_wav_file;
///
/// let path = Path::new("./tests/assets/sine.wav");
/// let file = OpenOptions::new().read(true).open(&path).unwrap();
/// let mut reader = BufReader::new(file);
/// let mut buf: Vec<u8> = Vec::new();
/// reader.read_to_end(&mut buf);
///
/// let (samples, num_samples) = samples_from_wave_bytes(buf).unwrap();
/// ```
pub fn samples_from_wave_bytes(bytes: Vec<u8>) -> Result<(Vec<f64>, usize)> {
    let mut cursor = Cursor::new(bytes);
    let wave = read_wav(&mut cursor)?;
    Ok(samples_from_wave(wave))
}

/// Given a path to a wave file, extract a `Vec<f64>` of samples from it and the size of that vec
///
/// ```
/// use synthrs::sample::samples_from_wave_file;
/// use synthrs::writer::read_wav_file;
///
/// let (samples, num_samples) = samples_from_wave_file("./tests/assets/sine.wav").unwrap();
/// ```
pub fn samples_from_wave_file(filepath: &str) -> Result<(Vec<f64>, usize)> {
    let wave = read_wav_file(filepath)?;
    Ok(samples_from_wave(wave))
}
