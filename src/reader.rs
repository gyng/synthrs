use std::cmp::max;
use std::path::Path;
use std::io::{ BufReader, Error, Result, Read, Seek, SeekFrom };
use std::fs::File;

use byteorder::{ BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt };

// http://www.midi.org/techspecs/midimessages.php
// http://www.ccarh.org/courses/253/handout/smf/
// http://www.ccarh.org/courses/253-2008/files/midifiles-20080227-2up.pdf
// http://dogsbodynet.com/fileformats/midi.html#RUNSTATUS

#[derive(NumFromPrimitive, PartialEq, Clone, Copy, Debug)]
pub enum MidiEventType {
    NoteOff = 0x8,
    NoteOn = 0x9,
    PolyponicKeyPressure = 0xa,
    ControlChange = 0xb,
    ProgramChange = 0xc,
    ChannelPressure = 0xd,
    PitchBendChange = 0xe,
    System = 0xf
}

#[derive(NumFromPrimitive, PartialEq, Clone, Copy, Debug)]
pub enum MidiSystemEventType {
    SystemExclusive = 0x0,
    TimeCodeQuaterFrame = 0x1,
    SongPositionPointer = 0x2,
    SongSelect = 0x3,
    TuneRequest = 0x6,
    EndOfSystemExclusive = 0x7,
    TimingClock = 0x8,
    Start = 0xa,
    Continue = 0xb,
    Stop = 0xc,
    ActiveSensing = 0xe,
    SystemResetOrMeta = 0xf
}

#[derive(NumFromPrimitive, PartialEq, Clone, Copy, Debug)]
pub enum MidiMetaEventType {
    SequenceNumber = 0x00,
    TextEvent = 0x01,
    CopyrightNotice = 0x02,
    SequenceOrTrackName = 0x03,
    InstrumentName = 0x04,
    LyricText = 0x05,
    MarkerText = 0x06,
    CuePoint = 0x07,
    MidiChannelPrefixAssignment = 0x20,
    EndOfTrack = 0x2f,
    TempoSetting = 0x51,
    SmpteOffset = 0x54,
    TimeSignature = 0x58,
    SequencerSpecificEvent = 0x7f
}

#[derive(Debug)]
pub struct MidiSong {
    pub max_time: usize,
    pub time_unit: isize,
    pub tracks: Vec<MidiTrack>,
    pub track_count: usize,
    pub bpm: f64
}

#[derive(Debug)]
pub struct MidiTrack {
    pub events: Vec<MidiEvent>,
    pub max_time: usize
}

impl MidiTrack {
    fn new() -> MidiTrack {
        let events: Vec<MidiEvent> = Vec::new();
        MidiTrack {
            events: events,
            max_time: 0
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MidiEvent {
    pub event_type: MidiEventType,
    pub system_event_type: Option<MidiSystemEventType>,
    pub meta_event_type: Option<MidiMetaEventType>,
    pub time: usize,
    pub delta_time: usize,
    pub channel: u8,
    pub value1: usize,
    pub value2: Option<usize>
}

struct EventIterator<'a, T> where T: Read+Seek+'a {
    reader: &'a mut T,
    time: usize,
    delta_time: usize,
    running_status: Option<MidiEventType>,
    running_channel: Option<u8>,
    is_running: bool,
    end_of_track: bool
}

enum DataLength { Single, Double }

impl<'a, T> EventIterator<'a, T> where T: Read+Seek+'a {
    fn new(reader: &'a mut T) -> EventIterator<'a, T> {
        EventIterator {
            reader: reader,
            time: 0,
            delta_time: 0,
            running_status: None,
            running_channel: None,
            is_running: false,
            end_of_track: false
        }
    }

    fn read_data_event(&mut self, length: DataLength) -> Result<MidiEvent> {
        // If running status is true, implicitly use previous event's status
        //
        // Normal:
        //   [Byte 1] [Byte 2] [Byte 3]
        //    Status   Data 1   Data 2
        //
        // Running status true:
        //   [Byte 1] [Byte 2]
        //    Data 1   Data 2

        if self.is_running {
            self.reader.seek(SeekFrom::Current(-1));
        }

        let (value1, value2) = match length {
            Single => (
                try!(self.reader.read_u8()) as usize,
                None
            ),
            Double => (
                try!(self.reader.read_u8()) as usize,
                Some(try!(self.reader.read_u8()) as usize)
            )
        };

        Ok(MidiEvent {
            event_type: self.running_status.unwrap(),
            system_event_type: None,
            meta_event_type: None,
            time: self.time,
            delta_time: self.delta_time,
            channel: self.running_channel.unwrap(),
            value1: value1,
            value2: value2
        })
    }

    /// Returns none if no system messages were handled
    fn read_system_event(&mut self) -> Result<Option<MidiEvent>> {
        let system_event_type: MidiSystemEventType = num::FromPrimitive::from_u8(self.running_channel.unwrap()).unwrap();
        let meta_data_size = try!(read_variable_number(self.reader));

        Ok(None)
    }
}

pub fn read_midi(filename: &str) -> Result<MidiSong> {
    let path = Path::new(filename);
    let file = File::open(&path).unwrap();
    let mut reader = BufReader::new(file);
    let mut song = try!(read_midi_header(&mut reader));

    for _ in 0usize..song.track_count {
        song.tracks.push(try!(read_midi_track(&mut reader)));
    }

    Ok(song)
}

fn read_midi_header<T>(reader: &mut T) -> Result<MidiSong> where T: Read+Seek {
    assert_eq!(try!(reader.read_u32::<BigEndian>()), 0x4d546864); // MThd in hexadecimal
    assert_eq!(try!(reader.read_u32::<BigEndian>()), 6);          // Header length; always 6 bytes
    let _file_format  = try!(reader.read_u16::<BigEndian>());     // 0 = single track, 1 = multitrack, 2 = multisong
    let track_count   = try!(reader.read_u16::<BigEndian>());
    let time_division = try!(reader.read_u16::<BigEndian>());     // If positive, units per beat. If negative, SMPTE units

    Ok(MidiSong {
        max_time: 0,
        time_unit: time_division as isize,
        tracks: Vec::new(),
        track_count: track_count as usize,
        bpm: 120.0 // MIDI default BPM, can be changed by MIDI events later
    })
}

fn read_midi_track<T>(reader: &mut T) -> Result<MidiTrack> where T: Read+Seek {
    assert_eq!(try!(reader.read_u32::<BigEndian>()), 0x4d54726b); // MTrk in hexadecimal
    let _track_chunk_size = try!(reader.read_u32::<BigEndian>());
    let mut track = MidiTrack::new();

    let event_iterator = EventIterator::new(reader);
    track.events = event_iterator.map(|event| {
        event.unwrap()
    }).collect::<Vec<_>>();

    track.max_time = if track.events.len() > 1 {
        track.events[track.events.len() - 1usize].time
    } else {
        0
    };

    Ok(track)
}

fn read_variable_number<T>(reader: &mut T) -> Result<usize> where T: Read+Seek {
    // http://en.wikipedia.org/wiki/Variable-length_quantity
    // cont. bit---V
    //             7[6 5 4 3 2 1 0]+-+
    // more bytes: 1 b b b b b b b   | concat bits to form new number
    //                               V
    //                             7[6 5 4 3 2 1 0]
    //              no more bytes: 0 b b b b b b b

    let mut octet = try!(reader.read_u8());
    let mut value = (octet & 0b01111111) as usize;
    while octet >= 0b10000000 {
        octet = try!(reader.read_u8());
        value = (value << 7) as usize + (octet & 0b01111111) as usize;
    }

    Ok(value)
}

#[test]
fn it_parses_a_midi_file() {
    let song = read_midi("tests/assets/test.mid").ok().expect("Failed");

    assert_eq!(song.tracks.len(), 2); // metadata track included
    let ref messages = song.tracks[1].events;

    // ProgramChange
    assert_eq!(messages[0].event_type, MidiEventType::ProgramChange);
    assert_eq!(messages[0].time, 0);
    assert_eq!(messages[0].channel, 0);
    assert_eq!(messages[0].value1, 0);
    assert_eq!(messages[0].value2, None);

    // NoteOn
    assert_eq!(messages[1].event_type, MidiEventType::NoteOn);
    assert_eq!(messages[1].time, 0);
    assert_eq!(messages[1].channel, 0);
    assert_eq!(messages[1].value1, 57);
    assert_eq!(messages[1].value2, Some(64));

    // NoteOff
    assert_eq!(messages[2].event_type, MidiEventType::NoteOff);
    assert_eq!(messages[2].time, 960);
    assert_eq!(messages[2].channel, 0);
    assert_eq!(messages[2].value1, 57);
    assert_eq!(messages[2].value2, Some(0));
}

#[test]
fn it_parses_a_midi_file_with_multiple_tracks() {
    let song = read_midi("tests/assets/multitrack.mid").unwrap();
    assert_eq!(song.tracks.len(), 3);
}

#[test]
fn it_parses_a_midi_file_with_running_status() {
    let song = read_midi("tests/assets/running_status.mid").unwrap();
    assert_eq!(song.tracks.len(), 1);
    assert_eq!(song.max_time, 5640);
}

#[test]
fn it_parses_the_bpm_of_a_midi_file() {
    let song = read_midi("tests/assets/running_status.mid").unwrap();
    assert_eq!(song.bpm as usize, 160);
}
