//! Writes audio samples to files. (Reads too, this module needs a rename)
//!
//! ### Audacity PCM import settings
//!
//! * File > Import > Raw Data...
//! * Signed 16-bit PCM
//! * Little-endian
//! * 1 Channel (Mono)
//! * Sample rate: 44_100Hz (or whatever your samples generated have)

use std::fs::OpenOptions;
use std::io::{BufReader, Error, Read, Result, Write};
use std::path::Path;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

/// Creates a file at `filename` and writes a bunch of `&[i16]` samples to it as a PCM file.
/// See module documentation for PCM settings.
///
/// ```
/// use synthrs::wave::sine_wave;
/// use synthrs::writer::write_pcm_file;
/// use synthrs::synthesizer::{quantize_samples, make_samples};
///
/// write_pcm_file(
///     "out/sine.wav",
///     &quantize_samples::<i16>(&make_samples(0.1, 44_100, sine_wave(440.0))),
/// ).expect("failed to write wav");
/// ```
pub fn write_pcm_file(filename: &str, samples: &[i16]) -> Result<()> {
    let path = Path::new(filename);
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&path)?;
    write_pcm(&mut f, samples)
}

/// Writes a bunch of `&[i16]` samples to a `Write` as raw PCM.
/// See module documentation for PCM settings.
///
/// Also see `synthrs::writer::write_pcm_file`.
///
/// ```
/// use std::io::Cursor;
/// use synthrs::wave::sine_wave;
/// use synthrs::writer::write_pcm;
/// use synthrs::synthesizer::{quantize_samples, make_samples};
///
/// let mut output_buffer: Vec<u8> = Vec::new();
/// let mut output_writer = Cursor::new(output_buffer);
///
/// // You can use whatever implements `Write`, such as a `File`.
/// write_pcm(
///     &mut output_writer,
///     &quantize_samples::<i16>(&make_samples(0.1, 44_100, sine_wave(440.0))),
/// ).expect("failed to write wav");
/// ```
pub fn write_pcm<W>(writer: &mut W, samples: &[i16]) -> Result<()>
where
    W: Write,
{
    for &sample in samples.iter() {
        writer.write_i16::<LittleEndian>(sample)?;
    }

    Ok(())
}

/// Creates a file at `filename` and writes a bunch of `&[i16]` samples to it as a WAVE file
/// ```
/// use synthrs::wave::sine_wave;
/// use synthrs::writer::write_wav_file;
/// use synthrs::synthesizer::{quantize_samples, make_samples};
///
/// write_wav_file(
///     "out/sine.wav",
///     44_100,
///     &quantize_samples::<i16>(&make_samples(0.1, 44_100, sine_wave(440.0))),
/// ).expect("failed to write wav");
/// ```
pub fn write_wav_file(filename: &str, sample_rate: usize, samples: &[i16]) -> Result<()> {
    let path = Path::new(filename);
    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&path)?;
    write_wav(&mut f, sample_rate, samples)
}

/// Writes a bunch of `&[i16]` samples to a `Write`. Also see `synthrs::writer::write_wav_file`.
/// ```
/// use std::io::Cursor;
/// use synthrs::wave::sine_wave;
/// use synthrs::writer::write_wav;
/// use synthrs::synthesizer::{quantize_samples, make_samples};
///
/// let mut output_buffer: Vec<u8> = Vec::new();
/// let mut output_writer = Cursor::new(output_buffer);
///
/// // You can use whatever implements `Write`, such as a `File`.
/// write_wav(
///     &mut output_writer,
///     44_100,
///     &quantize_samples::<i16>(&make_samples(0.1, 44_100, sine_wave(440.0))),
/// ).expect("failed to write wav");
/// ```
pub fn write_wav<W>(writer: &mut W, sample_rate: usize, samples: &[i16]) -> Result<()>
where
    W: Write,
{
    // See: http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/WAVE/WAVE.html
    // Some WAV header fields
    let channels = 1;
    let bit_depth = 16;
    let subchunk_2_size = samples.len() * channels * bit_depth / 8;
    let chunk_size = 36 + subchunk_2_size as i32;
    let byte_rate = (sample_rate * channels * bit_depth / 8) as i32;
    let block_align = (channels * bit_depth / 8) as i16;

    writer.write_i32::<BigEndian>(0x5249_4646)?; // ChunkID, RIFF
    writer.write_i32::<LittleEndian>(chunk_size)?; // ChunkSize
    writer.write_i32::<BigEndian>(0x5741_5645)?; // Format, WAVE

    writer.write_i32::<BigEndian>(0x666d_7420)?; // Subchunk1ID, fmt
    writer.write_i32::<LittleEndian>(16)?; // Subchunk1Size, 16 for PCM
    writer.write_i16::<LittleEndian>(1)?; // AudioFormat, PCM = 1 (linear quantization)
    writer.write_i16::<LittleEndian>(channels as i16)?; // NumChannels
    writer.write_i32::<LittleEndian>(sample_rate as i32)?; // SampleRate
    writer.write_i32::<LittleEndian>(byte_rate)?; // ByteRate
    writer.write_i16::<LittleEndian>(block_align)?; // BlockAlign
    writer.write_i16::<LittleEndian>(bit_depth as i16)?; // BitsPerSample

    writer.write_i32::<BigEndian>(0x6461_7461)?; // Subchunk2ID, data
    writer.write_i32::<LittleEndian>(subchunk_2_size as i32)?; // Subchunk2Size, number of bytes in the data

    for sample in samples {
        writer.write_i16::<LittleEndian>(*sample)?
    }

    Ok(())
}

// Borrowing of packed &wave.pcm is unsafe
// #[repr(C, packed)]
/// Representation of a WAV file. Does not contain fields for extended WAV formats.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Wave {
    pub chunk_id: i32,
    pub chunk_size: i32,
    pub format: i32,
    pub subchunk_1_id: i32,
    pub subchunk_1_size: i32,
    /// 1 = PCM
    pub audio_format: i16,
    pub num_channels: i16,
    pub sample_rate: i32,
    pub byte_rate: i32,
    pub block_align: i16,
    pub bits_per_sample: i16,
    pub subchunk_2_id: i32,
    pub subchunk_2_size: i32,
    pub pcm: Vec<i16>,
}

/// Reads a wave file given a file path. Convenience wrapper around `crate::writer::read_wav_file`.
/// ```
/// use synthrs::writer;
///
/// let wave = writer::read_wav_file("./tests/assets/sine.wav").unwrap();
/// assert_eq!(wave.num_channels, 1);
/// ```
pub fn read_wav_file(filename: &str) -> Result<Wave> {
    let path = Path::new(filename);
    let file = OpenOptions::new().read(true).open(&path)?;
    let mut reader = BufReader::new(file);
    read_wav(&mut reader)
}

/// Reads a wave file. Only supports mono 16-bit, little-endian, signed PCM WAV files
///
/// ### Useful commands:
///
/// * Use `ffmpeg -i .\example.wav` to inspect a wav
/// * Use `ffmpeg -i "dirty.wav" -f wav -flags +bitexact -acodec pcm_s16le -ar 44100 -ac 1 "clean.wav"` to clean a WAV
///
/// ```
/// use std::path::Path;
/// use std::fs::OpenOptions;
/// use std::io::BufReader;
/// use synthrs::writer;
///
/// let path = Path::new("./tests/assets/sine.wav");
/// let file = OpenOptions::new().read(true).open(&path).unwrap();
/// let mut reader = BufReader::new(file);
///
/// let wave = writer::read_wav(&mut reader).unwrap();
/// assert_eq!(wave.num_channels, 1);
/// ```
// For -acodec values, see https://trac.ffmpeg.org/wiki/audio%20types
pub fn read_wav<R>(reader: &mut R) -> Result<Wave>
where
    R: Read,
{
    let chunk_id = reader.read_i32::<BigEndian>()?; // ChunkID, RIFF
    let chunk_size = reader.read_i32::<LittleEndian>()?; // ChunkSize
    let format = reader.read_i32::<BigEndian>()?; // Format, WAVE
    if format != 0x5741_5645 {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "file is not a WAV".to_string(),
        ));
    }

    let subchunk_1_id = reader.read_i32::<BigEndian>()?; // Subchunk1ID, fmt
    let subchunk_1_size = reader.read_i32::<LittleEndian>()?; // Subchunk1Size, Chunk size: 16, 18 or 40

    let audio_format = reader.read_i16::<LittleEndian>()?; // AudioFormat, PCM = 1 (linear quantization)
    if audio_format != 1 {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "only 16-bit little-endian signed PCM WAV supported, audio_format: {}",
                audio_format
            ),
        ));
    }

    let num_channels = reader.read_i16::<LittleEndian>()?; // NumChannels
    let sample_rate = reader.read_i32::<LittleEndian>()?; // SampleRate
    let byte_rate = reader.read_i32::<LittleEndian>()?; // ByteRate
    let block_align = reader.read_i16::<LittleEndian>()?; // BlockAlign
    let bits_per_sample = reader.read_i16::<LittleEndian>()?; // BitsPerSample

    let extra_bytes = subchunk_1_size - 16;

    if extra_bytes > 0 {
        let extension_size = reader.read_i16::<LittleEndian>()?; // Size of the extension (0 or 22)
        if extension_size == 22 {
            // TODO: Add this to `Wave` as optionals
            let _valid_bits_per_sample = reader.read_i16::<LittleEndian>()?;
            let _speaker_mask = reader.read_i16::<LittleEndian>()?;
            let _subformat = reader.read_i16::<BigEndian>()?;
        } else if extension_size != 0 {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "unexpected fmt chunk extension size: should be 0 or 22 but got: {}",
                    extension_size
                ),
            ));
        }
    }

    let subchunk_2_id = reader.read_i32::<BigEndian>()?; // Subchunk2ID, data
    if subchunk_2_id != 0x6461_7461 {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "unexpected subchunk name: expecting data, got: {}",
                subchunk_2_id
            ),
        ));
    }
    let subchunk_2_size = reader.read_i32::<LittleEndian>()?; // Subchunk2Size, number of bytes in the data

    let mut pcm: Vec<i16> = Vec::with_capacity(2 * subchunk_2_size as usize);

    // `reader.read_into_i16(&pcm)` doesn't seem to work here due to bad input?
    // It just does nothing and &pcm is left empty after that.
    for _i in 0..subchunk_2_size {
        let sample = reader.read_i16::<LittleEndian>().unwrap_or(0);
        pcm.push(sample);
    }

    let wave = Wave {
        chunk_id,
        chunk_size,
        format,
        subchunk_1_id,
        subchunk_1_size,
        audio_format,
        num_channels,
        sample_rate,
        byte_rate,
        block_align,
        bits_per_sample,
        subchunk_2_id,
        subchunk_2_size,
        pcm,
    };

    Ok(wave)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_wav() {
        // 1 second at 44,100Hz, PCM 16 (2-byte)
        let wave = read_wav_file("./tests/assets/sine.wav").unwrap();

        assert_eq!(wave.format, 0x5741_5645); // WAVE
        assert_eq!(wave.audio_format, 1);
        assert_eq!(wave.num_channels, 1);
        assert_eq!(wave.sample_rate, 44_100);
        assert_eq!(wave.byte_rate, 88_200);
        assert_eq!(wave.block_align, 2);
        assert_eq!(wave.bits_per_sample, 16);
        assert_eq!(wave.subchunk_2_size, 88_200);
        assert_eq!(wave.pcm.len(), 88_200);
    }

    #[test]
    fn test_write_read_wav() {
        use crate::synthesizer::{make_samples, quantize_samples};
        use crate::wave::sine_wave;
        use crate::writer::write_wav;
        use std::io::{Cursor, Seek, SeekFrom};

        let output_buffer: Vec<u8> = Vec::new();
        let mut output_writer = Cursor::new(output_buffer);

        write_wav(
            &mut output_writer,
            44_100,
            &quantize_samples::<i16>(&make_samples(0.1, 44_100, sine_wave(440.0))),
        )
        .unwrap();

        let _ = output_writer.seek(SeekFrom::Start(0));
        let wave = read_wav(&mut output_writer).unwrap();
        assert_eq!(wave.format, 0x5741_5645); // WAVE
        assert_eq!(wave.audio_format, 1);
        assert_eq!(wave.num_channels, 1);
        assert_eq!(wave.sample_rate, 44_100);
        assert_eq!(wave.byte_rate, 88_200);
        assert_eq!(wave.block_align, 2);
        assert_eq!(wave.bits_per_sample, 16);
        assert_eq!(wave.subchunk_2_size, 8820);
        assert_eq!(wave.pcm.len(), 8820);
    }
}
