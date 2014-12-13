use std::io::{BufferedReader, File, IoResult};

// http://www.midi.org/techspecs/midimessages.php
// http://www.ccarh.org/courses/253/handout/smf/
// http://www.ccarh.org/courses/253-2008/files/midifiles-20080227-2up.pdf

// #[repr(u8)]
// enum MidiMessageType {
//     NoteOff = 0x8,
//     NoteOn = 0x9,
//     PolyponicKeyPressure = 0xa,
//     ControlChange = 0xb,
//     ProgramChange = 0xc,
//     ChannelPressure = 0xd,
//     PitchBendChange = 0xe,
//     System = 0xf
// }

#[deriving(Copy, Show)]
pub struct MidiMessage {
    message_type: u8, //MidiMessageType,
    time: uint,
    channel: u8,
    value1: u8,
    value2: Option<u8>
}

#[deriving(Show)]
pub struct MidiTrack {
    messages: Vec<MidiMessage>
}

impl MidiTrack {
    fn new() -> MidiTrack {
        let messages: Vec<MidiMessage> = Vec::new();
        MidiTrack {
            messages: messages
        }
    }
}

#[deriving(Show)]
pub struct MidiSong {
    time_unit: int,
    tracks: Vec<MidiTrack>,
    track_count: int
}

pub fn read_midi(filename: &str) -> IoResult<MidiSong> {
    let path = Path::new(filename);
    let mut file = BufferedReader::new(File::open(&path));

    // Header
    let _chunk_name = try!(file.read_be_u32()); // MThd
    let _chunk_size = try!(file.read_be_u32());
    let _file_format = try!(file.read_be_u16());
    let track_count = try!(file.read_be_u16());
    let ticks_per_quarter_note = try!(file.read_be_u16());

    let tracks: Vec<MidiTrack> = Vec::new();
    let mut song = MidiSong {
        time_unit: ticks_per_quarter_note as int,
        tracks: tracks,
        track_count: track_count as int
    };

    // Track chunk
    let _track_chunk_name = try!(file.read_be_u32()); // MTrk
    let _track_chunk_size = try!(file.read_be_u32());

    // Track data
    let mut track = MidiTrack::new();

    let mut keep_reading = true;

    while keep_reading {
        let delta_time = read_variable_number(&mut file).unwrap();

        let next_byte = try!(file.read_byte());
        let message_type = next_byte >> 4;
        let channel = next_byte & 0b00001111;

        // TODO: use an enum for this
        match message_type {
            8 | 9 | 10 | 11 | 14 => {
                track.messages.push(MidiMessage {
                    message_type: message_type,
                    time: delta_time,
                    channel: channel,
                    value1: try!(file.read_byte()),
                    value2: Some(try!(file.read_byte()))
                });
            },
            12 | 13 => {
                track.messages.push(MidiMessage {
                    message_type: message_type,
                    time: delta_time,
                    channel: channel,
                    value1: try!(file.read_byte()),
                    value2: None
                });
            },
            15 => {
                let system_message_type = try!(file.read_byte());
                let system_data_size = try!(read_variable_number(&mut file));

                match system_message_type {
                    0x2f => {
                        println!("End of track");
                        song.tracks.push(track);
                        track = MidiTrack::new();

                        let next_track_chunk_name = file.read_be_u32();
                        if next_track_chunk_name.is_ok() {
                            let _next_track_chunk_size = try!(file.read_be_u32());
                        } else {
                            keep_reading = false;
                        }
                    },
                    _ => {
                        // Discard other system messages
                        try!(file.read_exact(system_data_size as uint));
                    }
                }
            },
            _ => {
                panic!("message is not of a recognized MIDI message type (corrupt or unknown file format?)");
                // return Err((), "message is not of a recognized MIDI message type (corrupt or unknown file format?)");
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
    assert_eq!(messages[0].message_type, 12);
    assert_eq!(messages[0].time, 0);
    assert_eq!(messages[0].channel, 0);
    assert_eq!(messages[0].value1, 0);
    assert_eq!(messages[0].value2, None);

    // NoteOn
    assert_eq!(messages[1].message_type, 9);
    assert_eq!(messages[1].time, 0);
    assert_eq!(messages[1].channel, 0);
    assert_eq!(messages[1].value1, 57);
    assert_eq!(messages[1].value2, Some(64));

    // NoteOff
    assert_eq!(messages[2].message_type, 8);
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
