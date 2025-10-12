// use std::fs::{File, metadata};
// use std::io::{self, BufReader, Read};
// use std::path::Path;

// pub fn gentest(input_file_path: impl AsRef<Path>, output_file_path: impl AsRef<Path>) -> io::Result<()> {
//    //  let input_file = File::open(&input_file_path)?;
//    //  // let mut reader = io::BufReader::new(&input_file);
//    //  //
//    //  // // let mut output_file = File::create(output_file_path)?;
//    //  //
//    //  // for line in reader.lines() {
//    //  //     let line = line?;
//    //  //     if line.to_lowercase().contains("insert into") {
//    //  //         break;
//    //  //     }
//    //  //     dbg!(&line);
//    //  //     // writeln!(output_file, "{}", line)?;
//    //  // }
//    //
//    //  let length: usize = metadata(&input_file_path)
//    //      .expect("Unable to query file details")
//    //      .len()
//    //      .try_into()
//    //      .expect("Couldn't convert len from u64 to usize");
//    //
//    //  let mut reader = BufReader::new(input_file);
//    //
//    //
//    // const BLOCK_SIZE: usize = 2_097_152; //2M
//    //  let mut contents = vec![0_u8; BLOCK_SIZE];
//    //  let mut read_length: usize = 0;
//    //  for _ in 0..=(length / BLOCK_SIZE) {
//    //      // We don't want to read lines for this experiment to keep it
//    //      // consistent with the others even though that's the most common use of
//    //      // BufReader.
//    //      read_length += reader.read(&mut contents).expect("Couldn't read file");
//    //      let s = String::from_utf8(contents.clone()).unwrap();
//    //
//    //      let inside_string = false;
//    //      let last_closed = 0;
//    //      for c in s.chars() {
//    //          if c == '\'' {
//    //              inside_string
//    //          }
//    //      }
//    //  }

//     Ok(())
// }
