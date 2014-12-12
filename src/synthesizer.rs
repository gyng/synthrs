use std::num::{ Float, from_f64 };
use std::mem::size_of;

pub fn quantize<T>(input: f64) -> T where T: FromPrimitive {
    let quantization_levels = 2.0.powf(size_of::<T>() as f64 * 8.0) - 1.0;
    // Convert from [-1, 1] to take up full quantization range
    from_f64::<T>(input * (quantization_levels / 2.0)).expect("failed to quantize to given type")
}

pub fn quantize_samples<T>(input: Vec<f64>) -> Vec<T> where T: FromPrimitive {
    input.iter().map(|s| { quantize::<T>(*s) }).collect()
}

pub fn generate<F>(x: f64, f: F) -> f64 where F: Fn<(f64, ), f64> {
    f(x)
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
