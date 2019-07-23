
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use byteorder::{LittleEndian, WriteBytesExt};
//use serde::Deserialize;
//ReadBytesExt

use flatbuffers::{FlatBufferBuilder}; //, WIPOffset, Vector};

use crate::dvs_event_generated::event_cam;


pub fn csv_to_flatbuf(csv_path: &Path, flatbuf_path: &Path) {

  //open the out fb file for writing
  let outfile = File::create(flatbuf_path).expect("Couldn't open outfile");
  let mut outfile_writer = BufWriter::new(outfile);

  //open the csv file for reading
  let infile = File::open(csv_path).expect("Couldn't open infile");
  let infile_reader = BufReader::new(infile);

  let mut csv_reader = csv::ReaderBuilder::new()
    .delimiter(b' ')
    .from_reader(infile_reader);

  let mut rising_evt_count = 0;
  let mut falling_evt_count = 0;

  //create a new fbb on every chunk
  let mut fbb:FlatBufferBuilder = FlatBufferBuilder::new();
  let mut chunk_events:Vec<event_cam::ChangeEvent> = vec!();
  for result in csv_reader.records() {
    if result.is_ok() {
      let record = result.unwrap();
      //println!("{:?}", record);
      //events.txt: One event per line (timestamp x y polarity)
      //StringRecord(["0.706715000 58 68 1"])

      let timestamp = record[0].parse::<f64>().unwrap();
      let x_pos:u16 = record[1].parse::<u16>().unwrap();
      let y_pos:u16 = record[2].parse::<u16>().unwrap();
      let polarity:i8 = record[3].parse::<i8>().unwrap();

      //we currently only support rising and falling polarity
      if polarity > 0 { rising_evt_count += 1; }
      else { falling_evt_count +=1; }

      chunk_events.push(
        event_cam::ChangeEvent::new(
          timestamp.into(),
          x_pos.into(),
          y_pos.into(),
          polarity
        )
      );

      if chunk_events.len() == 100 {

        let chunk_data = flatten_framedata(&mut fbb, rising_evt_count, falling_evt_count, chunk_events.as_slice());
        let chunk_size = chunk_data.len();
        println!("chunk_size: {}", chunk_size);
        outfile_writer.write_u32::<LittleEndian>(chunk_size as u32);
        outfile_writer.write_all(chunk_data);

        //reset counts etc
        rising_evt_count = 0;
        falling_evt_count = 0;
        chunk_events.clear();
        fbb.reset();
      }

    }
  }


}



pub fn flatten_framedata<'a>(fbb: &'a mut FlatBufferBuilder, rising_count: u32, falling_count: u32, events:&[event_cam::ChangeEvent])
                             -> &'a[u8] {

//  println!("rising: {} falling: {} total: {}", rising_count, falling_count, events.len());

  let event_vector = fbb.create_vector(events);
  let root = {
    let mut fd_bldr = event_cam::FrameDataBuilder::new(fbb);
    fd_bldr.add_rising_count(rising_count);
    fd_bldr.add_falling_count(falling_count);
    fd_bldr.add_events(event_vector);
    fd_bldr.finish()
  };

  fbb.finish(root, None);
  fbb.finished_data()

}



#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
