use std::io::{BufferedReader, File, IoResult, IoError};

// http://www.midi.org/techspecs/midimessages.php
// http://www.ccarh.org/courses/253/handout/smf/
// http://www.ccarh.org/courses/253-2008/files/midifiles-20080227-2up.pdf
// http://dogsbodynet.com/fileformats/midi.html#RUNSTATUS

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
    pub bpm: f64
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
    pub system_event_type: Option<MidiSystemEventType>,
    pub meta_event_type: Option<MidiMetaEventType>,
    pub time: uint,
    pub delta_time: uint,
    pub channel: u8,
    pub value1: uint,
    pub value2: Option<uint>
}

struct MidiEventIterator<'a, T> where T: Reader+'a {
    reader: &'a mut T,
    time: uint,
    running_status: Option<MidiEventType>,
    running_channel: Option<u8>
}

impl<'a, T> MidiEventIterator<'a, T> where T: Reader+'a {
    fn new(reader: &'a mut T) -> MidiEventIterator<'a, T> {
        MidiEventIterator {
            reader: reader,
            time: 0,
            running_status: None,
            running_channel: None
        }
    }
}

// Similar to try! but this wraps the IoError return in an Option instead
macro_rules! try_some(
    ($e:expr) => (match $e { Ok(e) => e, Err(e) => return Some(Err(e)) })
)

// Drops unhandled messages
// TODO: Clean this mess up - all the mess is contained in here
impl<'a, T> Iterator<Result<MidiEvent, IoError>> for MidiEventIterator<'a, T> where T: Reader+'a {
    fn next(&mut self) -> Option<Result<MidiEvent, IoError>> {
        loop {
            let delta_time = try_some!(read_variable_number(self.reader));
            self.time += delta_time;

            let mut is_running = true;
            let next_byte = try_some!(self.reader.read_byte());

            if next_byte >= 0x80 {
                let event_type: MidiEventType = FromPrimitive::from_u8(next_byte >> 4).unwrap();
                self.running_status = Some(event_type);
                self.running_channel = Some(next_byte & 0b00001111);
                is_running = false;
            }

            match self.running_status.unwrap() {
                MidiEventType::NoteOff
                | MidiEventType::NoteOn
                | MidiEventType::PolyponicKeyPressure
                | MidiEventType::ControlChange
                | MidiEventType::PitchBendChange => {
                    return Some(Ok(MidiEvent {
                        event_type: self.running_status.unwrap(),
                        system_event_type: None,
                        meta_event_type: None,
                        time: self.time,
                        delta_time: delta_time,
                        channel: self.running_channel.unwrap(),
                        value1: (if is_running { next_byte } else { try_some!(self.reader.read_byte()) }) as uint,
                        value2: Some(try_some!(self.reader.read_byte()) as uint)
                    }))
                },

                MidiEventType::ProgramChange
                | MidiEventType::ChannelPressure => {
                    return Some(Ok(MidiEvent {
                        event_type: self.running_status.unwrap(),
                        system_event_type: None,
                        meta_event_type: None,
                        time: self.time,
                        delta_time: delta_time,
                        channel: self.running_channel.unwrap(),
                        value1: (if is_running { next_byte } else { try_some!(self.reader.read_byte()) }) as uint,
                        value2: None
                    }))
                },

                MidiEventType::System => {
                    // Handle Sysex messages
                    let system_event_type: MidiSystemEventType = FromPrimitive::from_u8(self.running_channel.unwrap()).unwrap();

                    match system_event_type {
                        MidiSystemEventType::SystemExclusive => {
                            let _ = read_sysex(self.reader); // sysex messages discarded
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
                            try_some!(self.reader.read_exact(2));
                        },

                        MidiSystemEventType::SystemResetOrMeta => {
                            // Typically these are meta messages
                            let meta_message_type: Option<MidiMetaEventType> = FromPrimitive::from_u8(try_some!(self.reader.read_byte()));
                            let meta_data_size = try_some!(read_variable_number(self.reader));

                            match meta_message_type {
                                Some(MidiMetaEventType::EndOfTrack) => {
                                    return None
                                },
                                Some(MidiMetaEventType::TempoSetting) => {
                                    assert_eq!(meta_data_size, 3u);
                                    let tempo_byte1 = if is_running { next_byte } else { try_some!(self.reader.read_byte()) };
                                    let tempo_byte2 = try_some!(self.reader.read_byte());
                                    let tempo_byte3 = try_some!(self.reader.read_byte());
                                    let tempo = (tempo_byte1 as uint << 16) as uint + (tempo_byte2 as uint << 8) as uint + tempo_byte3 as uint;

                                    return Some(Ok(MidiEvent {
                                        event_type: self.running_status.unwrap(),
                                        system_event_type: Some(system_event_type),
                                        meta_event_type: meta_message_type,
                                        time: self.time,
                                        delta_time: delta_time,
                                        channel: self.running_channel.unwrap(),
                                        value1: tempo,
                                        value2: None
                                    }))
                                },
                                _ => {
                                    // Discard unhandled meta messages
                                    try_some!(self.reader.read_exact(meta_data_size as uint));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
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

    // Guess song tempo (only take the first tempo change event)
    // This means tempo changes in-song are not supported
    for track in song.tracks.iter() {
        for event in track.messages.iter() {
            match event.meta_event_type {
                Some(MidiMetaEventType::TempoSetting) => {
                    song.bpm = (60000000.0 / event.value1 as f64) as f64;
                    break;
                },
                _ => {}
            }
        }
    }

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
        bpm: 120.0 // MIDI default BPM, can be changed by MIDI events later
    })
}

fn read_midi_track<T>(reader: &mut T) -> Result<MidiTrack, IoError> where T: Reader {
    // Track chunk header
    assert_eq!(try!(reader.read_be_u32()), 0x4d54726b); // MTrk in hexadecimal
    let _track_chunk_size = try!(reader.read_be_u32());

    let mut track = MidiTrack::new();
    // let mut previous_status: Option<MidiEventType> = None;

    // Read until end of track
    let mut event_iterator = MidiEventIterator::new(reader);

    // track.messages = event_iterator.collect();
    for event in event_iterator {
        track.messages.push(event.unwrap());
    }

    track.max_time = if track.messages.len() > 1 {
        track.messages[track.messages.len() - 1u].time
    } else {
        0
    };

    Ok(track)
}

fn read_sysex<T>(reader: &mut T) -> Result<(), IoError> where T: Reader {
    // Discard all sysex messages
    // Variable data length: read until EndOfSystemExclusive byte
    let mut next_byte = try!(reader.read_byte()) & 0b00001111;
    let mut system_event_type: Option<MidiSystemEventType> = FromPrimitive::from_u8(next_byte);
    while system_event_type != Some(MidiSystemEventType::EndOfSystemExclusive) {
        next_byte = try!(reader.read_byte()) & 0b00001111;
        system_event_type = FromPrimitive::from_u8(next_byte);
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
    assert_eq!(song.bpm as uint, 160);
}
