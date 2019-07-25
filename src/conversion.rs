
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Read};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use flatbuffers::{FlatBufferBuilder};

use crate::dvs_event_generated::event_cam;

use arcstar::sae_types::SaeEvent;

/// write the events in the given chunk to a file, with the chunk length prefix
fn write_chunk_to_file(fbb: &mut FlatBufferBuilder, outfile_writer:&mut BufWriter<File>, rising_evt_count: u32, falling_evt_count: u32, chunk_events:&[event_cam::ChangeEvent] ) {
  let chunk_data = flatten_framedata(fbb, rising_evt_count, falling_evt_count, chunk_events);
  let chunk_size = chunk_data.len();
  //println!("chunk_size: {}", chunk_size);
  outfile_writer.write_u32::<LittleEndian>(chunk_size as u32).expect("write failed");
  outfile_writer.write_all(chunk_data).expect("write failed");

  //reset counts etc
  fbb.reset();
}

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

  let mut fbb:FlatBufferBuilder = FlatBufferBuilder::new();
  let mut chunk_events:Vec<event_cam::ChangeEvent> = vec!();
  for result in csv_reader.records() {
    if result.is_ok() {
      let record = result.unwrap();
      //events.txt: One event per line (timestamp x y polarity) eg ["0.706715000 58 68 1"]

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
        write_chunk_to_file(&mut fbb, &mut outfile_writer, rising_evt_count, falling_evt_count, chunk_events.as_slice());
        //reset counts etc
        rising_evt_count = 0;
        falling_evt_count = 0;
        chunk_events.clear();
      }

    }
    else {
      // ensure that we flush any queued events when input finishes, or get another read error
      if chunk_events.len() > 0 {
        println!("final chunk events: {}", chunk_events.len());
        write_chunk_to_file(&mut fbb, &mut outfile_writer, rising_evt_count, falling_evt_count, chunk_events.as_slice());
      }
      break;
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



/// read the next set of events from the current length-prefixed chunk in the file
pub fn read_next_chunk_sae_events(buf_reader: &mut BufReader<File>) -> Vec<SaeEvent> {

  let chunk_len = buf_reader.read_u32::<LittleEndian>().unwrap_or(0);
  //println!("chunk_len: {}", chunk_len);

  if chunk_len > 0 {
    let mut buf: Vec<u8> = vec![0; chunk_len as usize];
    buf_reader.read_exact(&mut buf).expect("couldn't read_exact");
    let frame_data: event_cam::FrameData = flatbuffers::get_root::<event_cam::FrameData>(&buf);
    let fb_events = frame_data.events().expect("no events");

    let total_evts = fb_events.len();
    let sum_evts_check: usize = (frame_data.rising_count() + frame_data.falling_count()) as usize;
    if total_evts != sum_evts_check {
      eprintln!("MISMATCH total_evts {} rising {} falling {} ",
                total_evts, frame_data.rising_count(), frame_data.falling_count());
    }

    let event_list:Vec<SaeEvent> = fb_events.iter().map(|fb_event| {
      SaeEvent {
        row: fb_event.y() as u16,
        col: fb_event.x() as u16,
        polarity: fb_event.polarity() as u8,
        timestamp: fb_event.time() as u32,
        norm_descriptor: None,
      }
    }).collect();

    return event_list;
  }

  //otherwise return empty vector (no events)
  vec!()
}


//#[cfg(test)]
//mod tests {
//  #[test]
//  fn it_works() {
//    assert_eq!(2 + 2, 4);
//  }
//}
