use anyhow::{Error};
use pdf::{file::File};
use std::env::args;

mod transcriptparser;



// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }




fn main() -> Result<(), Error> {

    // $ cargo run -- ./test.pdf

    // let paths = fs::read_dir("./").unwrap();
    // for path in paths {
    //     println!("Name: {}", path.unwrap().path().display())
    // }

    let path = args().nth(1).expect("no file given");
    println!("read: {}", path);


    let pdf = File::<Vec<u8>>::open(&path).unwrap();

    let ret = transcriptparser::parse_pdf_transcript(pdf)
   

    Ok(())


}