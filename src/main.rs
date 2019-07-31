use std::path::Path;

use eventcam_converter::conversion;

#[macro_use]
extern crate clap;

/// Simple example of converting the CSV event format into a flatbuffer
fn main() {

  let matches = clap_app!(eventcam_converter =>
        (version: "0.1.0")
        (author: "Todd Stellanova")
        (about: "Converts text event files into flatbuffer binary format")
        (@arg INPUT: -i --input +takes_value  "Sets the input file to use")
        (@arg OUTPUT: -o --output +takes_value  "Sets the output file to use")
        (@arg CHUNK_SIZE: -c --chunksize +takes_value  "Num of events per chunk (default 1000)")
    ).get_matches();

  let infile = matches.value_of("INPUT").unwrap_or("./data/events.txt");
  let outfile = matches.value_of("OUTPUT").unwrap_or("./data/events.dat");
  let chunksize = matches.value_of("CHUNK_SIZE").unwrap_or("1000").parse::<usize>().unwrap();

  println!("Translate from {} to {}", infile, outfile);

  let in_path = Path::new(infile);
  let out_path = Path::new(outfile);
  let (record_count, chunk_count) = conversion::csv_to_flatbuf(&in_path, &out_path,chunksize);

  println!("Processed {} records , in {} chunks",record_count, chunk_count);

}
