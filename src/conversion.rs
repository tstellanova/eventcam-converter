
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Read};
use std::path::Path;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use flatbuffers::{FlatBufferBuilder};

use crate::dvs_event_generated::event_cam;

use arcstar::sae_types::{SaeEvent, SaeTime};
use crate::dvs_event_generated::event_cam::ChangeEvent;


///
/// Take a CSV events file, in the format described by
/// (Event-Camera Dataset and Simulator)[http://rpg.ifi.uzh.ch/davis_data.html]
/// and generate an equivalent flatbuffer file.
/// Briefly, the CSV format is:
/// One event per line, space delimited (timestamp x y polarity) eg `0.706715000 58 68 1`
///
/// Returns the tuple `(record_count, chunk_count)` where chunk count is the number of length-prefixed
/// chunk buffers in the file
///
/// chunk_size is the max number of events per chunk: the only hard limit is the
/// size of a flatbuffer, which can't exceed 2 GB
pub fn csv_to_flatbuf(csv_path: &Path, flatbuf_path: &Path, chunk_size: usize) -> (u32, u32) {

  //open the out fb file for writing
  let outfile = File::create(flatbuf_path).expect("Couldn't open flatbuf output file for write.");
  let mut outfile_writer = BufWriter::new(outfile);

  //open the csv file for reading
  let infile = File::open(csv_path).expect("Couldn't open CSV input file for read.");

  let mut csv_reader = csv::ReaderBuilder::new()
    .delimiter(b' ')
    .has_headers(false)
    .from_reader(infile) ;

  let mut chunk_count = 0;
  let mut rising_evt_count = 0;
  let mut falling_evt_count = 0;

  let mut fbb:FlatBufferBuilder = FlatBufferBuilder::new();
  let mut chunk_events:Vec<event_cam::ChangeEvent> = Vec::with_capacity(chunk_size);

  let mut csv_record:csv::StringRecord = csv::StringRecord::new();
  let mut record_count = 0;

  while let Ok(record_is_valid) = csv_reader.read_record(&mut csv_record) {
    if !record_is_valid {
      //this occurs when there are no more records to read
      break;
    }

    record_count += 1;
    let timestamp = csv_record[0].parse::<f64>().unwrap();
    let x_pos:u16 = csv_record[1].parse::<u16>().unwrap();
    let y_pos:u16 = csv_record[2].parse::<u16>().unwrap();
    let polarity:i8 = csv_record[3].parse::<i8>().unwrap();

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

    if chunk_events.len() == chunk_size {
      chunk_count += 1;
      write_chunk(&mut fbb, &mut outfile_writer, rising_evt_count, falling_evt_count, chunk_events.as_slice());
      //reset counts etc
      rising_evt_count = 0;
      falling_evt_count = 0;
      chunk_events.clear();
    }

  }

  // ensure that we flush any queued events when input finishes
  if chunk_events.len() > 0 {
    chunk_count += 1;
    write_chunk(&mut fbb, &mut outfile_writer, rising_evt_count, falling_evt_count, chunk_events.as_slice());
  }

  (record_count, chunk_count)
}


/// write the events in the given chunk to a stream, with the chunk length prefix
fn write_chunk<W: Write>(fbb: &mut FlatBufferBuilder, writer:&mut BufWriter<W>, rising_evt_count: u32, falling_evt_count: u32, chunk_events:&[event_cam::ChangeEvent] ) {
  let chunk_data = flatten_framedata(fbb, rising_evt_count, falling_evt_count, chunk_events);
  let chunk_size = chunk_data.len();
  //println!("chunk_size: {}", chunk_size);
  writer.write_u32::<LittleEndian>(chunk_size as u32).expect("write failed");
  writer.write_all(chunk_data).expect("write failed");

  // clear the data sitting in the FlatBufferBuilder
  fbb.reset();
}

fn flatten_framedata<'a>(fbb: &'a mut FlatBufferBuilder, rising_count: u32, falling_count: u32, events:&[event_cam::ChangeEvent])
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


fn convert_change_event_to_sae_event(fb_event: &ChangeEvent, timebase: f64, timescale: f64) -> SaeEvent {
  SaeEvent {
    row: fb_event.y() as u16,
    col: fb_event.x() as u16,
    polarity: fb_event.polarity() as u8,
    timestamp: ((fb_event.time() - timebase) /timescale) as SaeTime,
    norm_descriptor: None,
  }
}

/// Converts flatbuffer events from a Reader into SAE Events
/// read the next set of flatbuffer events from the current length-prefixed chunk in the stream
pub fn read_next_chunk_sae_events<R: Read>(buf_reader: &mut BufReader<R>, timebase: f64, timescale: f64) -> Option<Vec<SaeEvent>> {

  let chunk_len = buf_reader.read_u32::<LittleEndian>().unwrap_or(0);
  //println!("chunk_len: {}", chunk_len);

  if chunk_len > 0 {
    let mut buf: Vec<u8> = vec![0; chunk_len as usize];
    let read_res = buf_reader.read_exact(&mut buf);
    if read_res.is_err() {
      eprintln!("Events read error: {:?}", read_res);
      return None;
    }
    let frame_data: event_cam::FrameData = flatbuffers::get_root::<event_cam::FrameData>(&buf);
    let fb_events = frame_data.events().expect("no events");

    let total_evts = fb_events.len();
    let sum_evts_check: usize = (frame_data.rising_count() + frame_data.falling_count()) as usize;
    if total_evts != sum_evts_check {
      //TODO return an error result or Option instead
      eprintln!("MISMATCH total_evts {} rising {} falling {} ",
                total_evts, frame_data.rising_count(), frame_data.falling_count());
    }

    //TODO candidate for rayon par_iter ?
    let event_list:Vec<SaeEvent> = fb_events.iter().map( |fb_event|
      convert_change_event_to_sae_event(&fb_event, timebase, timescale)
    ).collect();

    return Some(event_list);
  }

  //otherwise return empty vector (no events)
  None
}


#[cfg(test)]
mod tests {
  use super::*;
//  use assert_approx_eq::assert_approx_eq;

  /// Convert event file from CSV to flatbuffer, then read back flatbuffer
  /// Caution: this test treats the `sample_25_events.txt` file as a gold file
  #[test]
  fn test_conversion_roundtrip() {

    let csv_path = Path::new("./data/sample_25_events.txt");
    let flatbuf_path = Path::new("./data/sample_25_events.dat");
    let (record_count, _chunk_count) = csv_to_flatbuf(&csv_path, &flatbuf_path, 1000);
    assert_eq!(record_count, 25);

    let infile = File::open(flatbuf_path).expect("Couldn't open flatbuf_path");
    let mut infile_reader = BufReader::new(infile);
    let timescale = 1E-6; //each tick of SaeTime is one microsecond ?
    let timebase = 0.0; //0.003811000 from gold file
    let event_list_opt = read_next_chunk_sae_events(&mut infile_reader, timebase, timescale);
    assert_eq!(true, event_list_opt.is_some());
    let event_list = event_list_opt.unwrap();
    assert_eq!(event_list.len(), 25);

    let event_slice = event_list.as_slice();
    let first_event = &event_slice[0];
    assert_eq!(first_event.timestamp, 3811);//value from gold file at given timescale

    let second_event_time = 0.003820001;
    let expected_time = ((second_event_time - timebase) / timescale) as SaeTime;
    let second_event = &event_slice[1];
    assert_eq!(expected_time, second_event.timestamp);


    //attempt to read more events from the reader-- should fail at EOF
    let event_list_opt2 = read_next_chunk_sae_events(&mut infile_reader, timebase, timescale);
    assert_eq!(true, event_list_opt2.is_none());

  }



}
