//! Functions for dealing with creating samples for sample-synthesis generators

use std::io::{Cursor, Result};

use crate::synthesizer::unquantize_samples;
use crate::writer::{read_wav, read_wav_file, Wave};

/// Given a `crate::writer::Wave`, extract a `Vec<f64>` of samples from it
///
/// ```
/// use synthrs::sampler::samples_from_wave;
/// use synthrs::writer::read_wav_file;
///
/// let wave = read_wav_file("./tests/assets/sine.wav").unwrap();
/// let samples = samples_from_wave(wave);
/// ```
pub fn samples_from_wave(wave: Wave) -> Vec<f64> {
    unquantize_samples(&wave.pcm)
}

/// Given a bunch of bytes for a wave file, extract a `Vec<f64>` of samples from it
pub fn samples_from_wave_bytes(bytes: Vec<u8>) -> Result<Vec<f64>> {
    let mut cursor = Cursor::new(bytes);
    let wave = read_wav(&mut cursor)?;
    Ok(samples_from_wave(wave))
}

/// Given a path to a wave file, extract a `Vec<f64>` of samples from it
///
/// ```
/// use synthrs::sampler::samples_from_wave_file;
/// use synthrs::writer::read_wav_file;
///
/// let samples = samples_from_wave_file("./tests/assets/sine.wav");
/// ```
pub fn samples_from_wave_file(filepath: &str) -> Result<Vec<f64>> {
    let wave = read_wav_file(filepath)?;
    Ok(samples_from_wave(wave))
}
