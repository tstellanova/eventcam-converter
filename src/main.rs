use std::path::Path;

use eventcam_converter::conversion;

/// Simple example of converting the CSV event format into a flatbuffer
fn main() {

  println!("Start!");

  let in_path = Path::new("./data/sample_120_events.txt");
  let out_path = Path::new("./data/sample_120_events.dat");
  let (record_count, chunk_count) = conversion::csv_to_flatbuf(&in_path, &out_path);

  println!("processed {} records , in {} chunks",record_count, chunk_count);

  println!("Done!");
}