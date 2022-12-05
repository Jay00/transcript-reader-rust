use anyhow::{Context, Error, Result};
use pdf::{file::File, content::Operation, primitive::Primitive, object::Page};
use std::{borrow::Cow,};
// use regex::Regex;

#[derive(Debug)]
pub struct PageSettings {
    pub starting_page_number: usize,
    pub margin_left_x: f32, // 0 points
    // pub margin_right_x: f32,
    pub line_number_limit_x: f32, // 112.0 points
    pub indent_left_postition_x: f32, // 200.0 points
    pub margin_bottom_y: f32, // 27.0 points
    pub margin_top_y: f32,
    pub margin_right_x: f32 // higher number
}

impl PageSettings {
    pub fn new(line_number_limit_x: f32, indent_left_position_x: f32) -> PageSettings {
        PageSettings { 
            starting_page_number: 1,
            margin_left_x: 0.0, 
            line_number_limit_x: line_number_limit_x, // 112.00
            indent_left_postition_x: indent_left_position_x, // 200.0
            margin_bottom_y: 27.0, 
            margin_top_y: 10000.00, 
            margin_right_x: 10000.00, 
        }
    }
}


#[derive(Debug, Clone)]
pub struct Line {
    pub page: usize,
    pub line: u32,
    pub text: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
}


#[derive(Debug, Clone, PartialEq)]
struct TextObject<'src> {
    pub x: f32,
    pub y: f32,
    pub text: Cow<'src, str>,
}


#[derive(Debug, Clone)]
struct TextObjectParser<'src> {
    ops: std::slice::Iter<'src, Operation>,
}




impl<'src> Iterator for TextObjectParser<'src> {
    type Item = TextObject<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut last_coords = None;
        let mut last_text = None;

        while let Some(Operation { operator, operands }) = self.ops.next() {
            println!("The Operation operator: {}, operand: {:?}", operator, operands);
            match (operator.as_str(), operands.as_slice()) {
                ("BT", _) => {
                    println!("Begin Text: Clear previous.");
                    // Clear all prior state because we've just seen a
                    // "begin text" op
                    last_coords = None;
                    last_text = None;
                }
                ("Td", [Primitive::Number(x), Primitive::Number(y)]) => {
                    // "Text Location" contains the location of the text on the
                    // current page.
                    last_coords = Some((*x, *y));
                    println!("Set last coords: {:?}", last_coords);
                }
                ("Tm", _) => {
                    // println!("The Operation operator: {}, operand: {:?}", operator, operands);
                    // print_type_of(operands);
                    println!("Operands Length: {}", operands.len());
                    // "Text Location" contains the location of the text on the
                    // current page.
                    // println!("Set last coords: ");

                    // for op in operands {
                    //     print_type_of(op);
                    //     let x = op.as_number().or(0.0)
                    // }

                    if operands.len() == 6{
                        last_coords = Some((operands[4].as_number().unwrap(), operands[5].as_number().unwrap()));
                        println!("Set last coords: {:?}", last_coords);
                    }

                    // last_coords = Some((*x, *y));
                }
                ("Tj", [Primitive::String(text)]) => {
                    // println!("The Operation operator: {}, operand: {:?}", operator, operands);
                    // print_type_of(operands);

                    // println!("Tj Setting text: {:?}", text.as_str().ok());
                    // "Show text" - the operation that actually contains the
                    // text to be displayed.
                    let t = text.as_str();
                    match t {
                        Result::Ok(txt) => {
                            println!("Show Text: Setting last text to: {}", txt);
                            last_text = Some(txt);
                        },
                        Result::Err(err) => {
                            panic!("PDF ERROR");
                        }
                    }


                    
                }
                ("TJ" | "Tj", [Primitive::Array(a)]) => {
                    // We have an array
                    // println!("TJ Found. Primitive: {:?}", a);
                    // println!("The Operation operator: {}, operand: {:?}", operator, operands);
                    // print_type_of(operands);
                  
                    let mut info = Vec::new();
         
                    for o in a {
                        match o {
                            Primitive::String(text) => {
                                // println!("Setting Text: {:?}", text);
                                // "Text Location" contains the location of the text on the
                                // current page.
                                // last_text = text.as_str().ok();
                                info.push(text.as_str().unwrap());
                            }
                            _ => continue
                        }
                    }

                    let combined_text = info.join("");
                    last_text = Some(Cow::from(combined_text));
                    
                }
                ("ET", _) => {
                    println!("End of Text");
                    // println!("Last Coordinates: {:?}", last_coords);
                    // "end of text" - we should have finished this text object,
                    // if we got all the right information then we can yield it
                    // to the caller. Otherwise, use take() to clear anything
                    // we've seen so far and continue.
                    if let (Some((x, y)), Some(text)) = (last_coords.take(), last_text.take()) {
                        println!("Yield TextObject TextObject: {{x: {}, y: {}, text: {} }}", x, y, text);
                        return Some(TextObject { x, y, text });
                    }

                }
                _ => continue,
            }
        }

        None
    }
}



fn text_objects(operations: &[Operation]) -> impl Iterator<Item = TextObject<'_>> + '_ {
    TextObjectParser {
        ops: operations.iter(),
    }
}


fn parse_text_objects_on_page<'a>(page: &'a Page, settings: &'a PageSettings, page_num: usize) -> Result<Vec<Line>, Error> {

    let content = match &page.contents {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };
    println!("Get text objects for Page {}", page_num);

    
    // Sort the PDF operations into lines of text
    let objects_sorted_into_lines = sort_text_objects_to_lines(&content.operations);


    // Merge Lines that are close together
    // Sometimes the Y axis value is very close, but not exact.
    // This can cause lines which appear to a human to be the same line
    // to be multiple primitive lines because the Y axis is off by 1 or 2 points
    // We should merge lines within a fuzzyness factor together to one line

    // Merge the Vectors of Primitive Lines within a few points of each other
    let fudge_factor = 5.0;
    let merged_lines = merge_lines(objects_sorted_into_lines, fudge_factor);



    // Convert the Primitive Lines to Lines
    let mut lines = Vec::with_capacity(merged_lines.capacity());
    for l in merged_lines {
        // println!("{:?}", l);
        let line = transform_to_line_object(l, page_num, settings);
        // println!("{:?}", line);
        // Ignore empty lines
        if line.line == 0 && line.text == None {
            // This is an empty line, do not push it onto the vector
            // println!("{:?} - NOT A Transcript LINE (skipped)", line);
        } else {
            // println!("{:?}", line);
            lines.push(line);
        }
     
    }
  
    Ok(lines)
}


fn sort_text_objects_to_lines<'a>(operations: &'a Vec<Operation>) -> Vec<Vec<TextObject<'a>>> {

    let mut text_objects = text_objects(&operations).collect::<Vec<TextObject>>();
    // Sort all text objects from top to bottom and then from left to right
    text_objects.sort_by(
        |a, b| 
        b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal)
    .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)));

    println!("Found {} text objects.", {text_objects.len()});
    // println!("Text Objects: {:?}", text_objects)


    // let re = Regex::new(r"^[A-Z \s \.]+:").unwrap();
    let mut primitive_lines = vec![];
    let mut last_y = None;
    let mut current_primitive_line = Vec::new();


    // Create the Primitive Lines objects and add them to the Vector
    for text_obj in text_objects {
        match last_y {
            Option::None => {
                // This is the first iteration of the page
                // println!("This is the first iteration of page {}", page_num);
                // println!("First line encountered at y: {}, text: {}", text_obj.y, text_obj.text);

                // Set the last Y
                last_y = Some(text_obj.y);
                // Create new Primitive Line Object
                current_primitive_line.push(text_obj);
            
                continue;
            },
            Option::Some(v) => {
                if v == text_obj.y {
                    // This is the same line as the last object
                    // println!("Same y position as last y: {}, text: {}", text_obj.y, text_obj.text);

                    current_primitive_line.push(text_obj);
                    
                } else {
                    // This is a new line as we have moved down the page
                    
                    // println!("New line encountered at y: {}, text: {}", text_obj.y, text_obj.text);

                    // Sort the line by the x value before we push it onto the vector
                    current_primitive_line.sort_by(
                        |a,b| 
                        b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal));


                    // println!("New line vector added: {:?}", current_primitive_line);
                    primitive_lines.push(current_primitive_line);

                    // We are done with the last vector and need to create a new vector
                    // Empty the current vector
                    current_primitive_line = Vec::new();

                    // Set the last Y
                    last_y = Some(text_obj.y);

                    // Push the new text_obj onto the new vector
                    current_primitive_line.push(text_obj);
                
                   

                    
                    continue;
                }
            }
            
        }
        
    }
    return primitive_lines;
}

fn merge_lines<'a>(lines: Vec<Vec<TextObject<'a>>>, fudge_factor: f32) -> Vec<Vec<TextObject>> {

    let mut merged_lines_vec = Vec::new();

    let mut last_y;
    let mut current_vec = Vec::new();
 

    match lines.get(0) {
        Option::Some(line_of_text_objs) => {

            current_vec.extend(line_of_text_objs.to_vec());
            let first_txt_obj = line_of_text_objs.get(0);

            match first_txt_obj {
                Option::Some(f) => {
                    last_y = f.y;
                },
                Option::None => {
                    // There is no object in the first array. This shouldn't happen.
                    panic!("There is no object in this array")
                },
            }
        },
        Option::None => {
            // No Lines on this page
            return merged_lines_vec;
        },
    }
    
    // Skip first line
    for this_line in lines[1..].iter() {
       let this_y = this_line.get(0).unwrap().y;

       let difference = last_y - this_y;

       if difference < fudge_factor {
        // This should be considered the same line
        // We need to extend the current_vec
        current_vec.extend(this_line.to_vec());

       } else {
        // This should be considered a new line

        // Create a new line and move to the merged array
        merged_lines_vec.push(current_vec);

        // Erase the previous and Create a new vector and use the current text objects to create it
        current_vec = this_line.to_vec();

        // Update the last_y position
        last_y = this_y;
       }
    }

    return merged_lines_vec;
}




fn transform_to_line_object(vector: Vec<TextObject>, page_number: usize, settings: &PageSettings) -> Line {
        
        let mut current_line_number = 0_u32;
        let mut current_line_text = None;
        let mut x = None;
        let mut y = None;

        for txt_obj in vector {
            // println!("Processing obj: {:?}", txt_obj);

            // Ignore everything outside the page margins
            if txt_obj.x < settings.margin_left_x {
                // Skip this object
                println!("Skipping object: OUTSIDE LEFT MARGIN");
                continue;
            }

            if txt_obj.x > settings.margin_right_x {
                 // Skip this object
                 println!("Skipping object: OUTSIDE RIGHT MARGIN");
                continue;
            }

            // Ignore Y value below bottom margin i.e, 27.0 points
            if txt_obj.y < settings.margin_bottom_y {
                 // Skip this object
                 println!("Skipping object: OUTSIDE BOTTOM MARGIN");
                continue;
            }

            // Ignore Y value above top margin, i.e. ?
            if txt_obj.y > settings.margin_top_y {
                 // Skip this object
                 println!("Skipping object: OUTSIDE TOP MARGIN");
                continue;
            }

            // Check if txt_object is in between the margin and line number limiter
            if txt_obj.x < settings.line_number_limit_x {
                // This is a line number on the side
                // println!("Possible Line number {}", txt_obj.text);
                // Attempt to parse the number
                let r = txt_obj.text.parse();
                match r {
                    // New Line Number
                    Ok(v) => {
                        current_line_number = v;
                    },
                    // Sometimes there is a blank space " " that is not parsable
                    // When this happens just leave the line number alone.
                    Err(_) => {},
                }
                continue;
            }

            // If the text object is to the left of the line number column
            // then this is text in the main page
            if txt_obj.x > settings.line_number_limit_x {
                // println!("Possible substantive transcript text found: {}", txt_obj.text);
                match current_line_text {
                    Option::None => {
                        // Nothing is yet set in the text line
                        let text = txt_obj.text.to_string();
                        if text.trim().is_empty() {
                            // ignore any white space text objects before actual text occurs
                            continue;
                        } else {
                            // This is the first text object containing actual text.
                            // We will use this text objects coordinates for the entire line
                            x = Some(txt_obj.x);
                            y = Some(txt_obj.y);
                            current_line_text = Some(text);
                        }
                    }
                    Option::Some(txt) => {
                        // We already have text set on the line
                        // concatenate this with whatever we have.
                        current_line_text = Some(txt + &txt_obj.text.to_string());
                    }
                }
                continue;
             
            }
        }

        // Create and return the tranformed line
        Line {
            page: page_number,
            line: current_line_number,
            text: current_line_text,
            x,
            y,
        }
}


pub fn parse_pdf_transcript(pdf: File<Vec<u8>>, settings: &PageSettings) -> Result<Vec<Line>, Error> {
    // Parse the complete pdf

    let mut lines = Vec::<Line>::new();

    for (i, page) in pdf.pages().enumerate(){
        // println!("Processing page: {}", i);
        let page = page?;
        let current_page_number = i + settings.starting_page_number;
        let lines_on_page = parse_text_objects_on_page(&page, settings, current_page_number)
        .with_context(|| format!("Unable to parse the members on page {}", i + 1))?;
        lines.extend(lines_on_page)
    }

    // for line in lines {
    //     println!("{:?}", line)
    // }

    Ok(lines)
}