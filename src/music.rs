use std::num::Float;

#[cfg(test)]
use std::num::FloatMath;

/// Equal-tempered
/// 0=C 1=C# 2=D 3=D# 4=E 5=F 6=F# 7=G 8=G# 9=A 10=B
pub fn note(a4: f64, semitone: uint, octave: uint) -> f64 {
    let semitones_from_a4 = octave as int * 12 + semitone as int - 9 - 48;
    a4 * (semitones_from_a4 as f64 * 2.0.ln() / 12.0).exp()
}

pub fn note_midi(a4: f64, midi_note: uint) -> f64 {
    let semitone = (midi_note - 24) % 12;
    let octave = ((midi_note - 24) / 12) + 1;
    note(a4, semitone, octave)
}

#[test]
fn it_equal_tempers() {
    let threshold = 0.1;
    let c4 = 261.63;
    let a4 = 440.0;
    let d3 = 146.83;
    let fs6 = 1479.98;
    assert!(FloatMath::abs_sub(note(a4, 9, 4), a4) < threshold);
    assert!(FloatMath::abs_sub(note(a4, 0, 4), c4) < threshold);
    assert!(FloatMath::abs_sub(note(a4, 2, 3), d3) < threshold);
    assert!(FloatMath::abs_sub(note(a4, 6, 6), fs6) < threshold);
}
