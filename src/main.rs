use midly::num::u7;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::Path;

struct Note {
    midi_note_number: Option<u7>,
    start_position: u32,
}

fn main() {
    let output_dir = Path::new("out");
    create_dir_all(&output_dir).unwrap();
    let mut w = BufWriter::new(File::create(output_dir.join("test.csv")).unwrap());
    writeln!(w, "NoteNumber,StartPosition,Length").unwrap();

    let data = std::fs::read("test.mid").unwrap();
    let smf = midly::Smf::parse(&data).unwrap();
    let mut note = Note {
        midi_note_number: None,
        start_position: 0,
    };

    println!("midi file has {} tracks!", smf.tracks.len());
    println!("tempo: {:?}", smf.header.timing);

    for tr in smf.tracks {
        let mut current_position: u32 = 0;
        for e in tr {
            current_position += e.delta.as_int();
            println!("{:?}, {}", e, current_position);
            if let midly::TrackEventKind::Midi { channel, message } = e.kind {
                match message {
                    //ノートオン
                    midly::MidiMessage::NoteOn { key, vel } => {
                        if let Some(n) = note.midi_note_number {
                            let length = current_position - note.start_position;
                            writeln!(
                                w,
                                "{},{},{}",
                                note.midi_note_number.unwrap(),
                                note.start_position,
                                length
                            )
                            .unwrap();
                            note.midi_note_number = None;
                        }

                        note.midi_note_number = Some(key);
                        note.start_position = current_position;
                    }
                    //ノートオフ
                    midly::MidiMessage::NoteOff { key, vel } => {
                        if note.midi_note_number.unwrap() == key {
                            let length = current_position - note.start_position;
                            writeln!(
                                w,
                                "{},{},{}",
                                note.midi_note_number.unwrap(),
                                note.start_position,
                                length
                            )
                            .unwrap();
                            note.midi_note_number = None;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
