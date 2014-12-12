use std::num::Float;

pub fn generate<F>(x: f64, f: F) -> f64 where F: Fn<(f64, ), f64> {
    f(x)
}

pub fn quantize_16(y: f64) -> i16 {
    // Quantization levels for 16 bits
    let levels = 2.0.powf(16.0) - 1.0;

    // Convert from [-1, 1] to [-2**16 / 2, 2**16 / 2]
    (y * (levels / 2.0)) as i16
}

pub fn quantize_sample_16(samples: Vec<f64>) -> Vec<i16> {
    samples.iter().map(|s| {
        quantize_16(*s)
    }).collect()
}

pub fn make_sample_16<F>(length: f64, sample_rate: uint, waveform: F) -> Vec<i16> where F: Fn<(f64, ), f64>+Copy {
    let num_samples = (sample_rate as f64 * length).floor() as uint;
    let mut samples: Vec<i16> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sample_rate as f64;
        samples.push(quantize_16(generate(t, waveform)));
    }

    samples
}

pub fn make_sample<F>(length: f64, sample_rate: uint, waveform: F) -> Vec<f64> where F: Fn<(f64, ), f64>+Copy {
    let num_samples = (sample_rate as f64 * length).floor() as uint;
    let mut samples: Vec<f64> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sample_rate as f64;
        samples.push(generate(t, waveform));
    }

    samples
}
