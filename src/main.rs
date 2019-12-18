#[macro_use]
extern crate clap;
use clap::{AppSettings};

fn main() {
    let arg_matches = clap_app!(lzss =>
       (about: "LZSS compression, decompriossion")
       (@subcommand encode =>
            (about: "Encode using LZSS algorithm")
            (@arg FILE: +required "File to encode")
            (@arg history_size: -s +takes_value default_value("12") "History window address size in bits")
            (@arg current_size: -c +takes_value default_value("16") "Current window address size in bits")
            (@arg search_depth: -d +takes_value default_value("0") 
                "Search depth for searching history for matches. 0 - all matches are found and the longest chosen. 1 - The first match is used")
       )
       (@subcommand decode =>
            (about: "Decode file encoded with this program")
            (@arg FILE: +required "File to decode")
       )
    ).setting(AppSettings::ArgRequiredElseHelp).get_matches();



    //    (@arg INPUT: +required "Sets the input file to use")
    //    (@arg debug: -d ... "Sets the level of debugging information")
    //    (@subcommand test =>
    //       (about: "controls testing features")
    //       (version: "1.3")
    //       (author: "Someone E. <someone_else@other.com>")
    //       (@arg verbose: -v --verbose "Print test information verbosely")
    //    )
}
