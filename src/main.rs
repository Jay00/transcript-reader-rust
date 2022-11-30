use anyhow::{Error};
use pdf::{file::File};
use std::env::args;

mod transcriptparser;



// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

struct PageSettings {
    pub margin_left_x: f32, // 112.0 points
    // pub margin_right_x: f32,
    pub indent_left_postition_x: f32, // 200.0 points
    pub margin_bottom_y: f32, // 27.0 points
    pub margin_top_y: f32,
}


fn main() -> Result<(), Error> {

    // $ cargo run -- ./test.pdf

    // let paths = fs::read_dir("./").unwrap();
    // for path in paths {
    //     println!("Name: {}", path.unwrap().path().display())
    // }

    let path = args().nth(1).expect("no file given");
    println!("read: {}", path);


    const MARGIN_X_LEFT: f32 = 112.0;
    const INDENT_X_LEFT: f32 = 200.0;

    let pdf = File::<Vec<u8>>::open(&path).unwrap();

    let ret = transcriptparser::parse_pdf_transcript(pdf);
   

    Ok(())


}