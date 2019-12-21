#[macro_use]
extern crate clap;
use clap::AppSettings;
use lzss::decoder;
use lzss::encoder::Encoder;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};

fn main() {
    let arg_matches = clap_app!(lzss =>
       (about: "LZSS compression, decompriossion")
       (version: "0.1.0")
       (@subcommand encode =>
            (about: "Encode using LZSS algorithm")
            (@arg FILE: +required "File to encode")
            (@arg ARCHIVE_PATH: +required "Resulting archive path")
            (@arg history_size: -s +takes_value default_value("12") "History window address size in bits")
            (@arg match_length_size: -c +takes_value default_value("4") "Match record length in bits. Determines size of the current window as well.")
            (@arg search_depth: -d +takes_value default_value("0") 
                "Search depth for searching history for matches. 0 - all matches are found and the longest chosen. 1 - The first match is used")
            (@arg overwrite: -o --overwrite "Overwrite existing file")
       )
       (@subcommand decode =>
            (about: "Decode file encoded with this program")
            (@arg ARCHIVE: +required "File to decode")
            (@arg FILE_PATH: +required "Resulting file path")
            (@arg overwrite: -o "Overwrite existing file")
       )
    ).setting(AppSettings::ArgRequiredElseHelp).get_matches();

    let subcommand_str = arg_matches
        .subcommand_name()
        .expect("Subcommand is required");
    if subcommand_str == "encode" {
        let sub_arg_matches = arg_matches.subcommand_matches("encode").unwrap();

        let filepath = sub_arg_matches.value_of("FILE").unwrap();
        let source_file = File::open(filepath).unwrap();
        let mut buff_reader = BufReader::new(source_file);

        let filepath = sub_arg_matches.value_of("ARCHIVE_PATH").unwrap();
        let dest_file = if sub_arg_matches.is_present("overwrite") {
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(filepath)
                .unwrap()
        } else {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(filepath)
                .unwrap()
        };
        let mut buff_writer = BufWriter::new(dest_file);

        let mut encoder = Encoder::new(
            sub_arg_matches
                .value_of("history_size")
                .unwrap()
                .parse()
                .expect("Unable to parse history_size"),
            sub_arg_matches
                .value_of("match_length_size")
                .unwrap()
                .parse()
                .expect("Unable to parse current_size"),
            sub_arg_matches
                .value_of("search_depth")
                .unwrap()
                .parse()
                .expect("Unable to parse search_depth"),
        );
        let res = encoder.encode(&mut buff_reader, &mut buff_writer);
        if res.is_err() {
            panic!("Error encoding: {}", res.err().unwrap());
        }
    } else if subcommand_str == "decode" {
        let sub_arg_matches = arg_matches.subcommand_matches("decode").unwrap();

        let filepath = sub_arg_matches.value_of("ARCHIVE").unwrap();
        let source_file = File::open(filepath).unwrap();
        let mut buff_reader = BufReader::new(source_file);

        let filepath = sub_arg_matches.value_of("FILE_PATH").unwrap();
        let dest_file = if sub_arg_matches.is_present("overwrite") {
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(filepath)
                .unwrap()
        } else {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(filepath)
                .unwrap()
        };
        let mut buff_writer = BufWriter::new(dest_file);

        let res = decoder::decode(&mut buff_reader, &mut buff_writer);
        if res.is_err() {
            panic!("Error decoding: {}", res.err().unwrap());
        }
    } else {
        panic!("Unknown command");
    }
}

// fn construct_archive_path(filepath: &str) -> String {
//     format!("{}.lzss", filepath)
// }
