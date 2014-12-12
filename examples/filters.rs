#![feature(unboxed_closures)]

extern crate synthrs;

use synthrs::{
    lowpass_filter, highpass_filter, bandpass_filter, bandreject_filter, make_sample,
    convolve, cutoff_from_frequency, SineWave, write_wav, make_sample_16, quantize_sample_16
};

fn main() {
    // Lowpass/highpass filter convolution example
    let sample = make_sample(1.0, 44100, |t: f64| -> f64 {
        0.33 * (SineWave(6000.0)(t) + SineWave(700.0)(t) + SineWave(80.0)(t))
    });

    let lowpass = lowpass_filter(cutoff_from_frequency(400.0, 44100), 0.01);
    write_wav("out/lowpass.wav", 44100,
        quantize_sample_16(sample.clone()) + quantize_sample_16(convolve(lowpass.clone(), sample.clone()))
    ).ok().expect("Failed");

    let highpass = highpass_filter(cutoff_from_frequency(2000.0, 44100), 0.01);
    write_wav("out/highpass.wav", 44100,
        quantize_sample_16(sample.clone()) + quantize_sample_16(convolve(highpass.clone(), sample.clone()))
    ).ok().expect("Failed");

    let bandpass = bandpass_filter(
        cutoff_from_frequency(500.0, 44100),
        cutoff_from_frequency(3000.0, 44100),
        0.01
    );
    write_wav("out/bandpass.wav", 44100,
        quantize_sample_16(sample.clone()) + quantize_sample_16(convolve(bandpass.clone(), sample.clone()))
    ).ok().expect("Failed");

    let bandreject = bandreject_filter(
        cutoff_from_frequency(400.0, 44100),
        cutoff_from_frequency(2000.0, 44100),
        0.01
    );
    write_wav("out/bandreject.wav", 44100,
        quantize_sample_16(sample.clone()) + quantize_sample_16(convolve(bandreject.clone(), sample.clone()))
    ).ok().expect("Failed");
}
