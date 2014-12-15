use std::io::{BufferedReader, File, IoResult};

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

#[deriving(Copy, Show)]
pub struct MidiEvent {
    pub message_type: MidiEventType,
    pub time: uint,
    pub channel: u8,
    pub value1: u8,
    pub value2: Option<u8>
}

#[deriving(Show)]
pub struct MidiTrack {
    pub messages: Vec<MidiEvent>
}

impl MidiTrack {
    fn new() -> MidiTrack {
        let messages: Vec<MidiEvent> = Vec::new();
        MidiTrack {
            messages: messages
        }
    }
}

#[deriving(Show)]
pub struct MidiSong {
    pub max_time: uint,
    pub time_unit: uint,
    pub tracks: Vec<MidiTrack>,
    pub track_count: uint
}

pub fn read_midi(filename: &str) -> IoResult<MidiSong> {
    let path = Path::new(filename);
    let mut file = BufferedReader::new(File::open(&path));

    // Header
    let _chunk_name = try!(file.read_be_u32()); // MThd
    let _chunk_size = try!(file.read_be_u32());
    let _file_format = try!(file.read_be_u16());
    let track_count = try!(file.read_be_u16());
    let time_division = try!(file.read_be_u16());

    if time_division & 0x8000 == 0 {
        // ticks per beat MIDI
    } else {
        // return Err("frames per second not supported");
        panic!("frames per second MIDI files are not supported")
    }

    let tracks: Vec<MidiTrack> = Vec::new();
    let mut song = MidiSong {
        max_time: 0,
        time_unit: time_division as uint,
        tracks: tracks,
        track_count: track_count as uint
    };

    // Track chunk
    let _track_chunk_name = try!(file.read_be_u32()); // MTrk
    let _track_chunk_size = try!(file.read_be_u32());

    // Track data
    let mut track = MidiTrack::new();
    let mut track_time = 0u;

    let mut keep_reading = true;

    while keep_reading {
        let delta_time = read_variable_number(&mut file).unwrap();
        track_time += delta_time;
        if track_time > song.max_time { song.max_time = track_time };

        let next_byte = try!(file.read_byte());
        let message_type: MidiEventType = FromPrimitive::from_u8(next_byte >> 4).unwrap();
        let channel = next_byte & 0b00001111;

        match message_type {
            MidiEventType::NoteOff
            | MidiEventType::NoteOn
            | MidiEventType::PolyponicKeyPressure
            | MidiEventType::ControlChange
            | MidiEventType::PitchBendChange => {
                track.messages.push(MidiEvent {
                    message_type: message_type,
                    time: track_time,
                    channel: channel,
                    value1: try!(file.read_byte()),
                    value2: Some(try!(file.read_byte()))
                });
            },
            MidiEventType::ProgramChange
            | MidiEventType::ChannelPressure => {
                track.messages.push(MidiEvent {
                    message_type: message_type,
                    time: track_time,
                    channel: channel,
                    value1: try!(file.read_byte()),
                    value2: None
                });
            },
            MidiEventType::System => {
                // Handle Sysex messages
                let system_message_type: MidiSystemEventType = FromPrimitive::from_u8(channel).unwrap();

                match system_message_type {
                    MidiSystemEventType::SystemExclusive => {
                        // Variable data length: read until EndOfSystemExclusive byte
                        let mut next_byte = try!(file.read_byte()) & 0b00001111;
                        let mut system_message_type: Option<MidiSystemEventType> = FromPrimitive::from_u8(next_byte);
                        while system_message_type != Some(MidiSystemEventType::EndOfSystemExclusive) {
                            next_byte = try!(file.read_byte()) & 0b00001111;
                            system_message_type = FromPrimitive::from_u8(next_byte);
                        }
                    },

                    MidiSystemEventType::TuneRequest
                    | MidiSystemEventType::TimingClock
                    | MidiSystemEventType::TimeCodeQuaterFrame
                    | MidiSystemEventType::Start
                    | MidiSystemEventType::Continue
                    | MidiSystemEventType::Stop
                    | MidiSystemEventType::ActiveSensing => {
                        // Unhandled, these have no data bytes
                        try!(file.read_byte());
                    },

                    MidiSystemEventType::SongPositionPointer
                    | MidiSystemEventType::SongSelect => {
                        // Unhandled, these have two data bytes
                        try!(file.read_byte());
                        try!(file.read_byte());
                    },

                    MidiSystemEventType::EndOfSystemExclusive => {
                        // All EndOfSystemExclusive messages should be captured by SystemExclusive message handling
                        panic!("unexpected EndOfSystemExclusive MIDI message: bad or corrupt file?")
                    },

                    MidiSystemEventType::SystemResetOrMeta => {
                        // Typically these are meta messages
                        let meta_message_type: Option<MidiMetaEventType> = FromPrimitive::from_u8(try!(file.read_byte()));
                        let meta_data_size = try!(read_variable_number(&mut file));

                        match meta_message_type {
                            Some(MidiMetaEventType::EndOfTrack) => {
                                song.tracks.push(track);
                                track = MidiTrack::new();
                                let next_track_chunk_name = file.read_be_u32();
                                if next_track_chunk_name.is_ok() {
                                    let _next_track_chunk_size = try!(file.read_be_u32());
                                    track_time = 0u;
                                } else {
                                    keep_reading = false;
                                }
                            },
                            _ => {
                                // Discard unhandled meta messages
                                try!(file.read_exact(meta_data_size as uint));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(song)
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
    assert_eq!(messages[0].message_type, MidiEventType::ProgramChange);
    assert_eq!(messages[0].time, 0);
    assert_eq!(messages[0].channel, 0);
    assert_eq!(messages[0].value1, 0);
    assert_eq!(messages[0].value2, None);

    // NoteOn
    assert_eq!(messages[1].message_type, MidiEventType::NoteOn);
    assert_eq!(messages[1].time, 0);
    assert_eq!(messages[1].channel, 0);
    assert_eq!(messages[1].value1, 57);
    assert_eq!(messages[1].value2, Some(64));

    // NoteOff
    assert_eq!(messages[2].message_type, MidiEventType::NoteOff);
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
