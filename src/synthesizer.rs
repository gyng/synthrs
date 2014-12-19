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

pub fn generate<F>(x: f64, f: &F) -> f64 where F: Fn<(f64, ), f64> {
    f.call((x, ))
}

pub fn make_samples<F>(length: f64, sample_rate: uint, waveform: F) -> Vec<f64> where F: Fn<(f64, ), f64> {
    let num_samples = (sample_rate as f64 * length).floor() as uint;
    let mut samples: Vec<f64> = Vec::with_capacity(num_samples);

    for i in range(0u, num_samples) {
        let t = i as f64 / sample_rate as f64;
        samples.push(generate(t, &waveform));
    }

    samples
}

pub fn peak_normalize(samples: Vec<f64>) -> Vec<f64> {
    let peak = samples.iter().fold(0.0f64, |acc, &sample| {
        acc.max(sample)
    });

    samples.iter().map(|&sample| {
        sample / peak
    }).collect()
}

// This is really awful, is there a more elegant way to do this?
pub fn make_samples_from_midi(sample_rate: uint, filename: &str) -> Vec<f64> {
    let song = reader::read_midi(filename).unwrap();
    let length = (60.0 * song.max_time as f64) / (song.bpm * song.time_unit as f64);

    let mut notes_on_for_ticks: Vec<Vec<(u8, u8)>> = Vec::new();
    for _ in range(0, song.max_time) {
        let notes_on_for_tick: Vec<(u8, u8)> = Vec::new();
        notes_on_for_ticks.push(notes_on_for_tick);
    }

    for track in song.tracks.iter() {
        for i in range(0, track.events.len()) {
            let event = track.events[i];
            if event.event_type == reader::MidiEventType::NoteOn {
                let start_tick = event.time;
                let note = event.value1;
                let velocity = event.value2.unwrap();

                let mut end_tick = song.max_time;
                for j in range(i, track.events.len()) {
                    let event_cursor = track.events[j];

                    // NoteOn with velocity 0 == NoteOff
                    if (event_cursor.event_type == reader::MidiEventType::NoteOff && event_cursor.value1 == note) ||
                       (event_cursor.event_type == reader::MidiEventType::NoteOn && event_cursor.value1 == note && event_cursor.value2.unwrap() == 0) {
                        end_tick = event_cursor.time;
                        break;
                    }
                }

                for tick in range(start_tick, end_tick) {
                    notes_on_for_ticks[tick].push((note as u8, velocity as u8));
                }
            }
        }
    }

    let midi_frequency_function = |t: f64| -> f64 {
        let tick = (t * song.bpm * song.time_unit as f64 / 60.0) as uint;
        let mut out = 0.0;

        if tick < notes_on_for_ticks.len() {
            for &(note, velocity) in notes_on_for_ticks[tick].iter() {
                let frequency = music::note_midi(440.0, note as uint);
                let loudness = (6.908 * (velocity as f64 / 255.0)).exp() / 1000.0;
                out += loudness * wave::SquareWave(frequency)(t)
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

    peak_normalize(samples)
}
