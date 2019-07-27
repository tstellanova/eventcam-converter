use std::path::Path;

use eventcam_converter::conversion;

#[macro_use]
extern crate clap;
use clap::{Arg, App, SubCommand};

/// Simple example of converting the CSV event format into a flatbuffer
fn main() {

  let matches = clap_app!(eventcam_converter =>
        (version: "0.1.0")
        (author: "Todd Stellanova")
        (about: "Converts text event files into flatbuffer binary format")
        (@arg INPUT: -i --input +takes_value  "Sets the input file to use")
        (@arg OUTPUT: -o --output +takes_value  "Sets the output file to use")
    ).get_matches();


  let infile = matches.value_of("input").unwrap_or("./data/events.txt");
  let outfile = matches.value_of("output").unwrap_or("./data/events.dat");
  println!("Translate from {} to {}", infile, outfile);

  let in_path = Path::new(infile);
  let out_path = Path::new(outfile);
  let (record_count, chunk_count) = conversion::csv_to_flatbuf(&in_path, &out_path);

  println!("processed {} records , in {} chunks",record_count, chunk_count);

  println!("Done!");
}