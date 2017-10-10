use std::cmp::max;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Result, Read, Seek, SeekFrom};
use std::path::Path;
use std::vec;

use byteorder::{BigEndian, ReadBytesExt};

// http://www.midi.org/techspecs/midimessages.php
// http://www.ccarh.org/courses/253/handout/smf/
// http://www.ccarh.org/courses/253-2008/files/midifiles-20080227-2up.pdf
// http://dogsbodynet.com/fileformats/midi.html#RUNSTATUS

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum EventType {
    NoteOff,
    NoteOn,
    PolyponicKeyPressure,
    ControlChange,
    ProgramChange,
    ChannelPressure,
    PitchBendChange,
    System,
}

impl EventType {
    fn from_u8(n: u8) -> Option<EventType> {
        match n {
            0x8 => Some(EventType::NoteOff),
            0x9 => Some(EventType::NoteOn),
            0xa => Some(EventType::PolyponicKeyPressure),
            0xb => Some(EventType::ControlChange),
            0xc => Some(EventType::ProgramChange),
            0xd => Some(EventType::ChannelPressure),
            0xe => Some(EventType::PitchBendChange),
            0xf => Some(EventType::System),
            _ => None,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SystemEventType {
    SystemExclusive,
    TimeCodeQuaterFrame,
    SongPositionPointer,
    SongSelect,
    TuneRequest,
    EndOfSystemExclusive,
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    SystemResetOrMeta,
}

impl SystemEventType {
    fn from_u8(n: u8) -> Option<SystemEventType> {
        match n {
            0x0 => Some(SystemEventType::SystemExclusive),
            0x1 => Some(SystemEventType::TimeCodeQuaterFrame),
            0x2 => Some(SystemEventType::SongPositionPointer),
            0x3 => Some(SystemEventType::SongSelect),
            0x6 => Some(SystemEventType::TuneRequest),
            0x7 => Some(SystemEventType::EndOfSystemExclusive),
            0x8 => Some(SystemEventType::TimingClock),
            0xa => Some(SystemEventType::Start),
            0xb => Some(SystemEventType::Continue),
            0xc => Some(SystemEventType::Stop),
            0xe => Some(SystemEventType::ActiveSensing),
            0xf => Some(SystemEventType::SystemResetOrMeta),
            _ => None,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MetaEventType {
    SequenceNumber,
    TextEvent,
    CopyrightNotice,
    SequenceOrTrackName,
    InstrumentName,
    LyricText,
    MarkerText,
    CuePoint,
    MidiChannelPrefixAssignment,
    EndOfTrack,
    TempoSetting,
    SmpteOffset,
    TimeSignature,
    SequencerSpecificEvent,
}

impl MetaEventType {
    fn from_u8(n: u8) -> Option<MetaEventType> {
        match n {
            0x00 => Some(MetaEventType::SequenceNumber),
            0x01 => Some(MetaEventType::TextEvent),
            0x02 => Some(MetaEventType::CopyrightNotice),
            0x03 => Some(MetaEventType::SequenceOrTrackName),
            0x04 => Some(MetaEventType::InstrumentName),
            0x05 => Some(MetaEventType::LyricText),
            0x06 => Some(MetaEventType::MarkerText),
            0x07 => Some(MetaEventType::CuePoint),
            0x20 => Some(MetaEventType::MidiChannelPrefixAssignment),
            0x2f => Some(MetaEventType::EndOfTrack),
            0x51 => Some(MetaEventType::TempoSetting),
            0x54 => Some(MetaEventType::SmpteOffset),
            0x58 => Some(MetaEventType::TimeSignature),
            0x7f => Some(MetaEventType::SequencerSpecificEvent),
            _ => None,
        }
    }
}

pub struct MidiSong {
    pub max_time: usize,
    pub time_unit: isize,
    pub tracks: Vec<MidiTrack>,
    pub track_count: usize,
    pub bpm: f64,
}

pub struct MidiTrack {
    pub events: Vec<MidiEvent>,
    pub max_time: usize,
}

impl MidiTrack {
    fn new() -> MidiTrack {
        let events: Vec<MidiEvent> = Vec::new();
        MidiTrack {
            events: events,
            max_time: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MidiEvent {
    pub event_type: EventType,
    pub system_event_type: Option<SystemEventType>,
    pub meta_event_type: Option<MetaEventType>,
    pub time: usize,
    pub channel: u8,
    pub value1: usize,
    pub value2: Option<usize>,
}

impl MidiEvent {
    // NoteOn with velocity 0 == NoteOff
    pub fn is_note_terminating(self) -> bool {
        (self.event_type == EventType::NoteOff) ||
            (self.event_type == EventType::NoteOn && !self.value2.is_none() &&
                 self.value2.unwrap() == 0)
    }
}

struct EventIterator<'a, T>
where
    T: Read + Seek + 'a,
{
    reader: &'a mut T,
    time: usize,
    delta_time: usize,
    running_status: Option<EventType>,
    running_channel: Option<u8>,
    is_running: bool,
    end_of_track: bool,
}

#[derive(Debug)]
enum DataLength {
    Single,
    Double,
    System,
}

// Similar to try! but this wraps the IoError return in an Option instead
// This is used for the Iterator impl which has a signature of -> Option<Result<T>>
// Standard try! macro returns a Result<T>, which is unsuited for usage in `fn next`
// try! cannot be used inside a function which returns Option<Result<T, IoError>>
macro_rules! try_opt(
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => return Some(Err(Error::new(ErrorKind::Other, format!("{}", e))))
    })
);

impl IntoIterator for MidiSong {
    type Item = MidiTrack;
    type IntoIter = vec::IntoIter<MidiTrack>;

    fn into_iter(self) -> vec::IntoIter<MidiTrack> {
        IntoIterator::into_iter(self.tracks)
    }
}

impl IntoIterator for MidiTrack {
    type Item = MidiEvent;
    type IntoIter = vec::IntoIter<MidiEvent>;

    fn into_iter(self) -> vec::IntoIter<MidiEvent> {
        IntoIterator::into_iter(self.events)
    }
}

impl<'a, T> EventIterator<'a, T>
where
    T: Read + Seek + 'a,
{
    fn new(reader: &'a mut T) -> EventIterator<'a, T> {
        EventIterator {
            reader: reader,
            time: 0,
            delta_time: 0,
            running_status: None,
            running_channel: None,
            is_running: false,
            end_of_track: false,
        }
    }

    fn read_data_event(&mut self, length: DataLength) -> Result<MidiEvent> {
        // If running status is true, implicitly use previous event's status
        //
        // Normal double-byte event:
        //   [Byte 1] [Byte 2] [Byte 3]
        //    Status   Data 1   Data 2
        //
        // Running status true:
        //   [Byte 1] [Byte 2]
        //    Data 1   Data 2

        let (value1, value2) = match length {
            DataLength::Single => (self.reader.read_u8()? as usize, None),
            DataLength::Double => (
                self.reader.read_u8()? as usize,
                Some(self.reader.read_u8()? as usize),
            ),
            DataLength::System => {
                panic!("this should not have happened");
            }
        };

        Ok(MidiEvent {
            event_type: self.running_status.unwrap(),
            system_event_type: None,
            meta_event_type: None,
            time: self.time,
            channel: self.running_channel.unwrap(),
            value1: value1,
            value2: value2,
        })
    }

    /// Returns none if no system messages were handled
    fn read_system_event(&mut self) -> Option<Result<MidiEvent>> {
        let system_event_type = SystemEventType::from_u8(self.running_channel.unwrap()).unwrap();

        match system_event_type {
            SystemEventType::SystemExclusive => {
                let _ = self.read_sysex(); // discard sysex messages
            }

            SystemEventType::EndOfSystemExclusive => {
                // All EndOfSystemExclusive messages should be captured by SystemExclusive message handling
                panic!(
                    "unexpected EndOfSystemExclusive MIDI message: unsupported, bad, or corrupt file?"
                )
            }

            SystemEventType::TuneRequest |
            SystemEventType::TimingClock |
            SystemEventType::TimeCodeQuaterFrame |
            SystemEventType::Start |
            SystemEventType::Continue |
            SystemEventType::Stop |
            SystemEventType::ActiveSensing => {
                // Unhandled, these have no data bytes
            }

            SystemEventType::SongPositionPointer |
            SystemEventType::SongSelect => {
                // Unhandled, these have two data bytes
                try_opt!(self.reader.seek(SeekFrom::Current(2)));
            }

            SystemEventType::SystemResetOrMeta => {
                // These are typically meta messages
                return self.read_meta_event(system_event_type);
            }
        }

        None
    }

    fn read_meta_event(&mut self, system_event_type: SystemEventType) -> Option<Result<MidiEvent>> {
        let meta_message_type = MetaEventType::from_u8(self.reader.read_u8().unwrap());
        let meta_data_size = try_opt!(self.read_variable_number());

        match meta_message_type {
            Some(MetaEventType::EndOfTrack) => {
                self.end_of_track = true;
            }

            Some(MetaEventType::TempoSetting) => {
                assert_eq!(meta_data_size, 3usize);
                let tempo_byte1 = try_opt!(self.reader.read_u8()) as usize;
                let tempo_byte2 = try_opt!(self.reader.read_u8()) as usize;
                let tempo_byte3 = try_opt!(self.reader.read_u8()) as usize;
                // Casting to usize below is done to kill shift overflow errors
                // Somehow, it works...
                let tempo = (tempo_byte1 << 16) as usize + (tempo_byte2 << 8) as usize +
                    tempo_byte3;

                return Some(Ok(MidiEvent {
                    event_type: self.running_status.unwrap(),
                    system_event_type: Some(system_event_type),
                    meta_event_type: meta_message_type,
                    time: self.time,
                    channel: self.running_channel.unwrap(),
                    value1: tempo,
                    value2: None,
                }));
            }

            _ => {
                // Discard unhandled meta messages
                try_opt!(self.reader.seek(SeekFrom::Current(meta_data_size as i64)));
            }
        }

        None
    }

    fn read_sysex(&mut self) -> Result<()> {
        // Discard all sysex messages
        // Variable data length: read until EndOfSystemExclusive byte
        let mut next_byte = self.reader.read_u8()? & 0b00001111;
        let mut system_event_type = SystemEventType::from_u8(next_byte);
        while system_event_type != Some(SystemEventType::EndOfSystemExclusive) {
            next_byte = self.reader.read_u8()? & 0b00001111;
            system_event_type = SystemEventType::from_u8(next_byte);
        }

        Ok(())
    }

    /// Returns (status, running channel)
    fn read_status_byte(&mut self) -> Option<(EventType, u8)> {
        let byte = self.reader.read_u8().unwrap();

        if byte >= 0x80 {
            let status = EventType::from_u8(byte >> 4).unwrap();
            let channel = byte & 0b00001111;
            Some((status, channel))
        } else {
            self.reader.seek(SeekFrom::Current(-1)).ok();
            None
        }
    }

    fn read_variable_number(&mut self) -> Result<usize> {
        // http://en.wikipedia.org/wiki/Variable-length_quantity
        // cont. bit---V
        //             7[6 5 4 3 2 1 0]+-+
        // more bytes: 1 b b b b b b b   | concat bits to form new number
        //                               V
        //                             7[6 5 4 3 2 1 0]
        //              no more bytes: 0 b b b b b b b

        let mut octet = self.reader.read_u8()?;
        let mut value = (octet & 0b01111111) as usize;
        while octet >= 0b10000000 {
            octet = self.reader.read_u8()?;
            value = (value << 7) as usize + (octet & 0b01111111) as usize;
        }

        Ok(value)
    }

    fn get_event_length(&self, event_type: EventType) -> DataLength {
        match event_type {
            EventType::NoteOff |
            EventType::NoteOn |
            EventType::PolyponicKeyPressure |
            EventType::ControlChange |
            EventType::PitchBendChange => DataLength::Double,

            EventType::ProgramChange |
            EventType::ChannelPressure => DataLength::Single,

            EventType::System => DataLength::System,
        }
    }
}

impl<'a, T> Iterator for EventIterator<'a, T>
where
    T: Read + Seek + 'a,
{
    type Item = Result<MidiEvent>;

    fn next(&mut self) -> Option<Result<MidiEvent>> {
        while !self.end_of_track {
            let delta_time = try_opt!(self.read_variable_number());
            self.time += delta_time;

            if let Some((status, channel)) = self.read_status_byte() {
                self.running_status = Some(status);
                self.running_channel = Some(channel);
                self.is_running = false;
            } else {
                self.is_running = true;
            }

            match self.get_event_length(self.running_status.unwrap()) {
                length @ DataLength::Single |
                length @ DataLength::Double => return Some(self.read_data_event(length)),
                DataLength::System => {
                    if let Some(system_event) = self.read_system_event() {
                        return Some(system_event);
                    }
                }
            }

            if self.is_running {
                try_opt!(self.reader.seek(SeekFrom::Current(-1)));
            }
        }

        None
    }
}

/// Reads from a MIDI file into a `Result<MidiSong>`.
pub fn read_midi<P: AsRef<Path>>(path: P) -> Result<MidiSong> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(err) => return Err(err),
    };
    let mut reader = BufReader::new(file);
    let mut song = read_midi_header(&mut reader)?;

    for _ in 0usize..song.track_count {
        song.tracks.push(read_midi_track(&mut reader)?);
    }

    song.max_time = song.tracks.iter().fold(0usize, |acc, track| {
        max(acc, track.max_time)
    });

    // Guess song tempo (only take the first tempo change event)
    // This means tempo changes in-song are not supported
    for track in &song.tracks {
        for event in &track.events {
            if let Some(MetaEventType::TempoSetting) = event.meta_event_type {
                song.bpm = (60000000.0 / event.value1 as f64) as f64;
                break;
            }
        }
    }

    Ok(song)
}

fn read_midi_header<T>(reader: &mut T) -> Result<MidiSong>
where
    T: Read + Seek,
{
    assert_eq!(reader.read_u32::<BigEndian>()?, 0x4d546864); // MThd in hexadecimal
    assert_eq!(reader.read_u32::<BigEndian>()?, 6); // Header length; always 6 bytes
    let _file_format = reader.read_u16::<BigEndian>()?; // 0 = single track, 1 = multitrack, 2 = multisong
    let track_count = reader.read_u16::<BigEndian>()?;
    let time_division = reader.read_u16::<BigEndian>()?; // If positive, units per beat. If negative, SMPTE units

    Ok(MidiSong {
        max_time: 0,
        time_unit: time_division as isize,
        tracks: Vec::new(),
        track_count: track_count as usize,
        bpm: 120.0, // MIDI default BPM, can be changed by MIDI events later
    })
}

fn read_midi_track<T>(reader: &mut T) -> Result<MidiTrack>
where
    T: Read + Seek,
{
    assert_eq!(reader.read_u32::<BigEndian>()?, 0x4d54726b); // MTrk in hexadecimal
    let _track_chunk_size = reader.read_u32::<BigEndian>()?;
    let mut track = MidiTrack::new();

    track.events = EventIterator::new(reader)
        .map(|event| event.unwrap())
        .collect::<Vec<_>>();

    track.max_time = if track.events.len() > 1 {
        track.events[track.events.len() - 1usize].time
    } else {
        0
    };

    Ok(track)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_a_midi_file() {
        let song = read_midi("tests/assets/test.mid").ok().expect("failed");

        assert_eq!(song.tracks.len(), 2); // metadata track included
        let ref messages = song.tracks[1].events;

        // ProgramChange
        assert_eq!(messages[0].event_type, EventType::ProgramChange);
        assert_eq!(messages[0].time, 0);
        assert_eq!(messages[0].channel, 0);
        assert_eq!(messages[0].value1, 0);
        assert_eq!(messages[0].value2, None);

        // NoteOn
        assert_eq!(messages[1].event_type, EventType::NoteOn);
        assert_eq!(messages[1].time, 0);
        assert_eq!(messages[1].channel, 0);
        assert_eq!(messages[1].value1, 57);
        assert_eq!(messages[1].value2, Some(64));

        // NoteOff
        assert_eq!(messages[2].event_type, EventType::NoteOff);
        assert_eq!(messages[2].time, 960);
        assert_eq!(messages[2].channel, 0);
        assert_eq!(messages[2].value1, 57);
        assert_eq!(messages[2].value2, Some(0));
    }

    #[test]
    fn it_parses_a_midi_file_with_multiple_tracks() {
        let song = read_midi("tests/assets/multitrack.mid").ok().expect(
            "failed",
        );
        assert_eq!(song.tracks.len(), 3);
    }

    #[test]
    fn it_parses_a_midi_file_with_running_status() {
        let song = read_midi("tests/assets/running_status.mid").ok().expect(
            "failed",
        );
        assert_eq!(song.tracks.len(), 1);
        assert_eq!(song.max_time, 5640);
    }

    #[test]
    fn it_parses_the_bpm_of_a_midi_file() {
        let song = read_midi("tests/assets/running_status.mid").ok().expect(
            "failed",
        );
        assert_eq!(song.bpm as usize, 160);
    }
}
