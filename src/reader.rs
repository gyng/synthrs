use std::io::{BufferedReader, File, IoResult, IoError};

// http://www.midi.org/techspecs/midimessages.php
// http://www.ccarh.org/courses/253/handout/smf/
// http://www.ccarh.org/courses/253-2008/files/midifiles-20080227-2up.pdf

#[repr(u8)]
#[deriving(FromPrimitive, PartialEq, Copy, Show)]
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

#[repr(u8)]
#[deriving(FromPrimitive, PartialEq, Copy, Show)]
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

#[repr(u8)]
#[deriving(FromPrimitive, PartialEq, Copy, Show)]
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

#[deriving(Show)]
pub struct MidiSong {
    pub max_time: uint,
    pub time_unit: int,
    pub tracks: Vec<MidiTrack>,
    pub track_count: uint,
    pub bpm: uint
}

#[deriving(Show)]
pub struct MidiTrack {
    pub messages: Vec<MidiEvent>,
    pub max_time: uint
}

impl MidiTrack {
    fn new() -> MidiTrack {
        let messages: Vec<MidiEvent> = Vec::new();
        MidiTrack {
            messages: messages,
            max_time: 0
        }
    }
}

#[deriving(Copy, Show)]
pub struct MidiEvent {
    pub event_type: MidiEventType,
    pub time: uint,
    pub delta_time: uint,
    pub channel: u8,
    pub value1: u8,
    pub value2: Option<u8>
}

pub fn read_midi(filename: &str) -> Result<MidiSong, IoError> {
    let path = Path::new(filename);
    let mut reader = BufferedReader::new(File::open(&path));
    let mut song = try!(read_midi_header(&mut reader));

    if !(song.time_unit & 0x8000 == 0) {
        panic!("unsupported time division format (SMPTE not supported)")
    }

    for _ in range(0u, song.track_count) {
        song.tracks.push(try!(read_midi_track(&mut reader)));
    }

    song.max_time = song.tracks.iter().fold(0u, |acc, track| {
        if track.max_time > acc { track.max_time } else { acc }
    });

    Ok(song)
}

fn read_midi_header<T>(reader: &mut T) -> IoResult<MidiSong> where T: Reader {
    assert_eq!(try!(reader.read_be_u32()), 0x4d546864) // MThd in hexadecimal
    assert_eq!(try!(reader.read_be_u32()), 6);         // Header length; always 6 bytes
    let _file_format = try!(reader.read_be_u16());     // 0 = single track, 1 = multitrack, 2 = multisong
    let track_count = try!(reader.read_be_u16());
    let time_division = try!(reader.read_be_u16());    // If positive, units per beat. If negative, SMPTE units

    Ok(MidiSong {
        max_time: 0,
        time_unit: time_division as int,
        tracks: Vec::new(),
        track_count: track_count as uint,
        bpm: 120 // MIDI default BPM, can be changed by MIDI events later
    })
}

fn read_midi_track<T>(reader: &mut T) -> Result<MidiTrack, IoError> where T: Reader {
    // Track chunk header
    assert_eq!(try!(reader.read_be_u32()), 0x4d54726b); // MTrk in hexadecimal
    let _track_chunk_size = try!(reader.read_be_u32());

    let mut track = MidiTrack::new();
    // let mut previous_status: Option<MidiEventType> = None;

    // Read until end of track
    loop {
        let delta_time = try!(read_variable_number(reader)) as uint;
        track.max_time += delta_time;
        let next_byte = try!(reader.read_byte());
        let event_type: MidiEventType = FromPrimitive::from_u8(next_byte >> 4).unwrap();
        // previous_status = Some(event_type);
        let channel = next_byte & 0b00001111;

        match event_type {
            MidiEventType::NoteOff
            | MidiEventType::NoteOn
            | MidiEventType::PolyponicKeyPressure
            | MidiEventType::ControlChange
            | MidiEventType::PitchBendChange => {
                track.messages.push(MidiEvent {
                    event_type: event_type,
                    time: track.max_time,
                    delta_time: delta_time,
                    channel: channel,
                    value1: try!(reader.read_byte()),
                    value2: Some(try!(reader.read_byte()))
                })
            },

            MidiEventType::ProgramChange
            | MidiEventType::ChannelPressure => {
                track.messages.push(MidiEvent {
                    event_type: event_type,
                    time: track.max_time,
                    delta_time: delta_time,
                    channel: channel,
                    value1: try!(reader.read_byte()),
                    value2: None
                })
            },

            MidiEventType::System => {
               // Handle Sysex messages
               let system_message_type: MidiSystemEventType = FromPrimitive::from_u8(channel).unwrap();

               match system_message_type {
                   MidiSystemEventType::SystemExclusive => {
                       let _ = read_sysex(reader); // sysex messages discarded
                   },

                   MidiSystemEventType::EndOfSystemExclusive => {
                       // All EndOfSystemExclusive messages should be captured by SystemExclusive message handling
                       panic!("unexpected EndOfSystemExclusive MIDI message: unsupported, bad, or corrupt file?")
                   },

                   MidiSystemEventType::TuneRequest
                   | MidiSystemEventType::TimingClock
                   | MidiSystemEventType::TimeCodeQuaterFrame
                   | MidiSystemEventType::Start
                   | MidiSystemEventType::Continue
                   | MidiSystemEventType::Stop
                   | MidiSystemEventType::ActiveSensing => {
                       // Unhandled, these have no data bytes
                   },

                   MidiSystemEventType::SongPositionPointer
                   | MidiSystemEventType::SongSelect => {
                       // Unhandled, these have two data bytes
                       try!(reader.read_exact(2));
                   },

                   MidiSystemEventType::SystemResetOrMeta => {
                       // Typically these are meta messages
                       let meta_message_type: Option<MidiMetaEventType> = FromPrimitive::from_u8(try!(reader.read_byte()));
                       let meta_data_size = try!(read_variable_number(reader));

                       match meta_message_type {
                            Some(MidiMetaEventType::EndOfTrack) => {
                                break
                            },
                            _ => {
                                 // Discard unhandled meta messages
                                try!(reader.read_exact(meta_data_size as uint));
                            }
                       }
                   }
               }
            }
        }
    }

    Ok(track)
}

fn read_sysex<T>(reader: &mut T) -> Result<(), IoError> where T: Reader {
    // Discard all sysex messages
    // Variable data length: read until EndOfSystemExclusive byte
    let mut next_byte = try!(reader.read_byte()) & 0b00001111;
    let mut system_message_type: Option<MidiSystemEventType> = FromPrimitive::from_u8(next_byte);
    while system_message_type != Some(MidiSystemEventType::EndOfSystemExclusive) {
        next_byte = try!(reader.read_byte()) & 0b00001111;
        system_message_type = FromPrimitive::from_u8(next_byte);
    }

    Ok(())
}

fn read_variable_number<T>(reader: &mut T) -> IoResult<uint> where T: Reader {
    // http://en.wikipedia.org/wiki/Variable-length_quantity
    // cont. bit---V
    //             7[6 5 4 3 2 1 0]+-+
    // more bytes: 1 b b b b b b b   | concat bits to form new number
    //                               V
    //                             7[6 5 4 3 2 1 0]
    //              no more bytes: 0 b b b b b b b

    let mut octet = try!(reader.read_byte());
    let mut value = (octet & 0b01111111) as uint;
    while octet >= 0b10000000 {
        octet = try!(reader.read_byte());
        value = (value << 7) as uint + (octet & 0b01111111) as uint;
    }

    Ok(value)
}

#[test]
fn it_parses_a_midi_file() {
    let song = read_midi("tests/assets/test.mid").ok().expect("Failed");

    assert_eq!(song.tracks.len(), 2); // metadata track included
    let ref messages = song.tracks[1].messages;

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
fn it_parses_a_multitrack_midi_file() {
    let song = read_midi("tests/assets/multitrack.mid").unwrap();
    assert_eq!(song.tracks.len(), 3); // metadata track included
}
