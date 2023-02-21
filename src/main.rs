use clap::Parser;
use midly::num::u7;
use std::fs::{create_dir, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input standard MIDI file to convert
    input: std::path::PathBuf,
}

struct Note {
    midi_note_number: Option<u7>,
    start_position: u32,
}

/// 出力先フォルダを作成
fn create_export_folder(path: &Path) -> PathBuf {
    let mut export_path = path.to_path_buf();
    export_path.pop();
    export_path.push("export");
    let metadata = path.to_path_buf().metadata().unwrap();
    assert!(metadata.permissions().readonly());

    if !export_path.exists() {
        if let Err(e) = create_dir(export_path.as_path()) {
            eprintln!("Failed on create \"export\" dir : {}", e)
        }
    }
    export_path
}

/// PathがMIDIファイルかどうか
fn is_midi_file(path: &Path) {
    if let Some(x) = path.extension() {
        if x.is_empty() {
            eprintln!("Input file extension is must to be \"mid\" or \"smf\"");
            std::process::exit(1);
        }

        match x.to_str().unwrap() {
            "mid" => {}
            "MID" => {}
            "smf" => {}
            "SMF" => {}
            _ => {
                eprintln!("Input file extension is must to be \"wav\"");
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    let input = args.input.clone();
    if !input.exists() {
        eprintln!("{} is not found", input.display());
        std::process::exit(1);
    }

    is_midi_file(&input); //inputがMIDIファイルか確認
    let input = input.canonicalize().unwrap(); //絶対パスに変換
    let input_filename = args.input;
    let input_filename = input_filename.file_stem().unwrap().to_str().unwrap();

    // exportフォルダを作成
    let mut export_path = create_export_folder(&input);
    println!(
        "Export folder: {}",
        export_path.canonicalize().unwrap().display()
    );
    export_path.push(format!("{}.csv", input_filename)); //変換後のCSVファイル

    // SMFファイルのHeader情報などをprint
    let input = std::fs::read(input).unwrap();
    let smf = midly::Smf::parse(&input).unwrap();
    println!("SMF info");
    println!("MIDI file has {} tracks", smf.tracks.len());
    println!("Timing: {:?}", smf.header.timing);

    let mut note = Note {
        midi_note_number: None,
        start_position: 0,
    };

    let mut w = BufWriter::new(File::create(export_path).unwrap());
    writeln!(w, "NoteNumber,StartPosition,Length").unwrap();

    for tr in smf.tracks {
        let mut current_position: u32 = 0;
        for e in tr {
            current_position += e.delta.as_int();
            if let midly::TrackEventKind::Midi {
                channel: _channel,
                message,
            } = e.kind
            {
                match message {
                    //ノートオン
                    midly::MidiMessage::NoteOn { key, vel: _vel } => {
                        if let Some(n) = note.midi_note_number {
                            let length = current_position - note.start_position;
                            writeln!(w, "{},{},{}", n, note.start_position, length).unwrap();
                            note.midi_note_number = None;
                        }

                        note.midi_note_number = Some(key);
                        note.start_position = current_position;
                    }
                    //ノートオフ
                    midly::MidiMessage::NoteOff { key, vel: _vel } => {
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
