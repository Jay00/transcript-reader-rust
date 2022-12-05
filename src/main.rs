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


    // const MARGIN_X_LEFT: f32 = 112.0;
    // const INDENT_X_LEFT: f32 = 200.0;

    let pdf = File::<Vec<u8>>::open(&path).unwrap();

    let setttings = transcriptparser::PageSettings::new(112.0, 200.0);

    let result = transcriptparser::parse_pdf_transcript(pdf, &setttings);

    match result {
        Result::Ok(lines) => {
            println!("Extracted {} lines from the transcript. ", lines.len());
        }  
        Result::Err(err) => {
            println!("ERROR: {}", err.to_string())
        }
    }


   

    Ok(())


}