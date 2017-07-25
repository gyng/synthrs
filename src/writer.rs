//! Writes samples to files.
//!
//! See any example file in `examples` for usage.

use std::fs::OpenOptions;
use std::io::Result;
use std::path::Path;

use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

pub fn write_pcm(filename: &str, samples: &[i16]) -> Result<()> {
    let path = Path::new(filename);
    let mut f = try!(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
    );

    for &sample in samples.iter() {
        try!(f.write_i16::<LittleEndian>(sample));
    }

    Ok(())
}

// See: https://ccrma.stanford.edu/courses/422/projects/WaveFormat/
pub fn write_wav(filename: &str, sample_rate: usize, samples: &[i16]) -> Result<()> {
    let path = Path::new(filename);
    let mut f = try!(
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
    );

    // Some WAV header fields
    let channels = 1;
    let bit_depth = 16;
    let subchunk_2_size = samples.len() * channels * bit_depth / 8;
    let chunk_size = 36 + subchunk_2_size as i32;
    let byte_rate = (sample_rate * channels * bit_depth / 8) as i32;
    let block_align = (channels * bit_depth / 8) as i16;

    f.write_i32::<BigEndian>(0x52494646)?; // ChunkID, RIFF
    f.write_i32::<LittleEndian>(chunk_size)?; // ChunkSize
    f.write_i32::<BigEndian>(0x57415645)?; // Format, WAVE

    f.write_i32::<BigEndian>(0x666d7420)?; // Subchunk1ID, fmt
    f.write_i32::<LittleEndian>(16)?; // Subchunk1Size, 16 for PCM
    f.write_i16::<LittleEndian>(1)?; // AudioFormat, PCM = 1 (linear quantization)
    f.write_i16::<LittleEndian>(channels as i16)?; // NumChannels
    f.write_i32::<LittleEndian>(sample_rate as i32)?; // SampleRate
    f.write_i32::<LittleEndian>(byte_rate)?; // ByteRate
    f.write_i16::<LittleEndian>(block_align)?; // BlockAlign
    f.write_i16::<LittleEndian>(bit_depth as i16)?; // BitsPerSample

    f.write_i32::<BigEndian>(0x64617461)?; // Subchunk2ID, data
    f.write_i32::<LittleEndian>(subchunk_2_size as i32)?; // Subchunk2Size, number of bytes in the data

    for sample in samples {
        f.write_i16::<LittleEndian>(*sample)?
    }

    Ok(())
}
