extern crate synthrs;

use synthrs::filter::*;
use synthrs::synthesizer::{make_samples, quantize_samples};
use synthrs::wave::sine_wave;
use synthrs::writer::write_wav_file;

fn main() {
    // Lowpass/highpass filter convolution example
    let sample = make_samples(1.0, 44_100, |t: f64| -> f64 {
        0.33 * (sine_wave(6000.0)(t) + sine_wave(700.0)(t) + sine_wave(80.0)(t))
    });

    let lowpass = lowpass_filter(cutoff_from_frequency(400.0, 44_100), 0.01);
    let mut lowpass_samples = quantize_samples::<i16>(&sample);
    lowpass_samples.extend_from_slice(&*quantize_samples::<i16>(&convolve(&lowpass, &sample)));
    write_wav_file("out/lowpass.wav", 44_100, &lowpass_samples).expect("failed");

    let highpass = highpass_filter(cutoff_from_frequency(2000.0, 44_100), 0.01);
    let mut highpass_samples = quantize_samples::<i16>(&sample);
    highpass_samples.extend_from_slice(&*quantize_samples::<i16>(&convolve(&highpass, &sample)));
    write_wav_file("out/highpass.wav", 44_100, &highpass_samples).expect("failed");

    let bandpass = bandpass_filter(
        cutoff_from_frequency(500.0, 44_100),
        cutoff_from_frequency(3000.0, 44_100),
        0.01,
    );
    let mut bandpass_samples = quantize_samples::<i16>(&sample);
    bandpass_samples.extend_from_slice(&*quantize_samples::<i16>(&convolve(&bandpass, &sample)));
    write_wav_file("out/bandpass.wav", 44_100, &bandpass_samples).expect("failed");

    let bandreject = bandreject_filter(
        cutoff_from_frequency(400.0, 44_100),
        cutoff_from_frequency(2000.0, 44_100),
        0.01,
    );
    let mut bandreject_samples = quantize_samples::<i16>(&sample);
    bandreject_samples
        .extend_from_slice(&*quantize_samples::<i16>(&convolve(&bandreject, &sample)));
    write_wav_file("out/bandreject.wav", 44_100, &bandreject_samples).expect("failed");

    // Stateful filters
    let mut comb = Comb::new(0.2, 44_100, 0.5, 0.5, 0.5);
    let comb_samples: Vec<f64> = sample.clone().into_iter().map(|s| comb.tick(s)).collect();
    write_wav_file(
        "out/comb.wav",
        44_100,
        &*quantize_samples::<i16>(comb_samples.as_slice()),
    )
    .expect("failed");

    let mut allpass = AllPass::new(1.0, 44_100, 0.5);
    let allpass_samples: Vec<f64> = sample
        .clone()
        .into_iter()
        .map(|s| allpass.tick(s))
        .collect();
    write_wav_file(
        "out/allpass.wav",
        44_100,
        &*quantize_samples::<i16>(allpass_samples.as_slice()),
    )
    .expect("failed");
}
