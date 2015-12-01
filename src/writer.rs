use std::fs::OpenOptions;
use std::io::Result;
use std::path::Path;

use byteorder::{ BigEndian, LittleEndian, WriteBytesExt };

pub fn write_pcm(filename: &str, samples: Vec<i16>) -> Result<()> {
    let path = Path::new(filename);
    let mut f = try!(OpenOptions::new().write(true).truncate(true).create(true).open(&path));

    for &sample in samples.iter() {
        try!(f.write_i16::<LittleEndian>(sample));
    }

    Ok(())
}

// See: https://ccrma.stanford.edu/courses/422/projects/WaveFormat/
pub fn write_wav(filename: &str, sample_rate: usize, samples: Vec<i16>) -> Result<()> {
    let path = Path::new(filename);
    let mut f = try!(OpenOptions::new().write(true).truncate(true).create(true).open(&path));

    // Some WAV header fields
    let channels = 1;
    let bit_depth = 16;
    let subchunk_2_size = samples.len() * channels * bit_depth / 8;
    let chunk_size = 36 + subchunk_2_size as i32;
    let byte_rate = (sample_rate * channels * bit_depth / 8) as i32;
    let block_align = (channels * bit_depth / 8) as i16;

    try!(f.write_i32::<BigEndian>(0x52494646));                // ChunkID, RIFF
    try!(f.write_i32::<LittleEndian>(chunk_size));             // ChunkSize
    try!(f.write_i32::<BigEndian>(0x57415645));                // Format, WAVE

    try!(f.write_i32::<BigEndian>(0x666d7420));                // Subchunk1ID, fmt
    try!(f.write_i32::<LittleEndian>(16));                     // Subchunk1Size, 16 for PCM
    try!(f.write_i16::<LittleEndian>(1));                      // AudioFormat, PCM = 1 (linear quantization)
    try!(f.write_i16::<LittleEndian>(channels as i16));        // NumChannels
    try!(f.write_i32::<LittleEndian>(sample_rate as i32));     // SampleRate
    try!(f.write_i32::<LittleEndian>(byte_rate));              // ByteRate
    try!(f.write_i16::<LittleEndian>(block_align));            // BlockAlign
    try!(f.write_i16::<LittleEndian>(bit_depth as i16));       // BitsPerSample

    try!(f.write_i32::<BigEndian>(0x64617461));                // Subchunk2ID, data
    try!(f.write_i32::<LittleEndian>(subchunk_2_size as i32)); // Subchunk2Size, number of bytes in the data

    for sample in samples.iter() {
        try!(f.write_i16::<LittleEndian>(*sample))
    }

    Ok(())
}
