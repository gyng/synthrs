#![feature(unboxed_closures)]

extern crate synthrs;

use synthrs::synthesizer::{ make_samples, quantize_samples };
use synthrs::filter::{
    convolve, cutoff_from_frequency,
    lowpass_filter, highpass_filter, bandpass_filter, bandreject_filter
};
use synthrs::wave::SineWave;
use synthrs::writer::write_wav;


fn main() {
    // Lowpass/highpass filter convolution example
    let sample = make_samples(1.0, 44100, |t: f64| -> f64 {
        0.33 * (SineWave(6000.0)(t) + SineWave(700.0)(t) + SineWave(80.0)(t))
    });

    let lowpass = lowpass_filter(cutoff_from_frequency(400.0, 44100), 0.01);
    let mut lowpass_samples = quantize_samples::<i16>(sample.clone());
    lowpass_samples.extend_from_slice(&*quantize_samples::<i16>(convolve(lowpass, sample.clone())));
    write_wav("out/lowpass.wav", 44100, lowpass_samples).ok().expect("failed");

    let highpass = highpass_filter(cutoff_from_frequency(2000.0, 44100), 0.01);
    let mut highpass_samples = quantize_samples::<i16>(sample.clone());
    highpass_samples.extend_from_slice(&*quantize_samples::<i16>(convolve(highpass, sample.clone())));
    write_wav("out/highpass.wav", 44100, highpass_samples).ok().expect("failed");

    let bandpass = bandpass_filter(
        cutoff_from_frequency(500.0, 44100),
        cutoff_from_frequency(3000.0, 44100),
        0.01
    );
    let mut bandpass_samples = quantize_samples::<i16>(sample.clone());
    bandpass_samples.extend_from_slice(&*quantize_samples::<i16>(convolve(bandpass, sample.clone())));
    write_wav("out/bandpass.wav", 44100, bandpass_samples).ok().expect("failed");

    let bandreject = bandreject_filter(
        cutoff_from_frequency(400.0, 44100),
        cutoff_from_frequency(2000.0, 44100),
        0.01
    );
    let mut bandreject_samples = quantize_samples::<i16>(sample.clone());
    bandreject_samples.extend_from_slice(&*quantize_samples::<i16>(convolve(bandreject, sample.clone())));
    write_wav("out/bandreject.wav", 44100, bandreject_samples).ok().expect("failed");
}
