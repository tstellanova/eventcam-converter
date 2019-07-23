use std::path::Path;

use eventcam_converter::conversion::csv_to_flatbuf;


fn main() {

  println!("Start!");

  let in_path = Path::new("./data/events.txt");
  let out_path = Path::new("./data/events.dat");
  csv_to_flatbuf(&in_path, &out_path);

  println!("Done!");
}