use std::num::{ Float, FloatMath, from_f64 };
use std::mem::size_of;

use music;
use reader;
use wave;

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

pub fn make_samples<F>(length: f64, sample_rate: uint, waveform: F) -> Vec<f64> where F: Fn<(f64, ), f64>+Copy {
    let num_samples = (sample_rate as f64 * length).floor() as uint;
    let mut samples: Vec<f64> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sample_rate as f64;
        samples.push(generate(t, waveform));
    }

    samples
}

pub fn normalize(samples: Vec<f64>) -> Vec<f64> {
    let peak = samples.iter().fold(0.0f64, |acc, sample| {
        acc.max(*sample)
    });

    samples.iter().map(|sample| {
        *sample / peak
    }).collect()
}

// This is really awful, is there a more elegant way to do this?
// TODO: Split out volume normalization into a function
pub fn make_samples_from_midi(sample_rate: uint, bpm: f64, filename: &str) -> Vec<f64> {
    let song = reader::read_midi(filename).unwrap();
    let length = (60.0 * song.max_time as f64) / (bpm * song.time_unit as f64);

    let mut notes_on_for_ticks: Vec<Vec<(u8, u8)>> = Vec::new();
    for _ in range(0, song.max_time) {
        let notes_on_for_tick: Vec<(u8, u8)> = Vec::new();
        notes_on_for_ticks.push(notes_on_for_tick);
    }

    for track in song.tracks.iter() {
        for i in range(0, track.messages.len()) {
            let event = track.messages[i];
            if event.message_type == 9 {
                let from_tick = event.time;
                let note = event.value1;
                let velocity = event.value2.unwrap();

                let mut to_tick = song.max_time;
                for j in range(i, track.messages.len()) {
                    let event_to = track.messages[j];
                    if event_to.message_type == 8 && event_to.value1 == note {
                        to_tick = event_to.time;
                        break;
                    }
                }

                for tick in range(from_tick, to_tick) {
                    notes_on_for_ticks[tick].push((note, velocity));
                }
            }
        }
    }

    let midi_frequency_function = |t: f64| -> f64 {
        let tick = (t * bpm * song.time_unit as f64 / 60.0) as uint;
        let mut out = 0.0;

        if tick < notes_on_for_ticks.len() {
            for tup in notes_on_for_ticks[tick].iter() {
                let note = tup.val0();
                let velocity = tup.val1();
                let frequency = music::note_midi(440.0, note as uint);
                let loudness = (6.908 * (velocity as f64 / 255.0)).exp() / 1000.0;
                out += loudness * (wave::SquareWave(frequency)(t) + wave::SquareWave(frequency)(t)) / 2.0
            }
        }

        out
    };

    let num_samples = (sample_rate as f64 * length).floor() as uint;
    let mut samples: Vec<f64> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sample_rate as f64;
        samples.push(midi_frequency_function(t));
    }

    normalize(samples)
}
