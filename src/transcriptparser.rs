use anyhow::{Context, Error};
use pdf::{file::File, content::Operation, primitive::Primitive, object::Page};
use std::{iter::Peekable, marker::PhantomData, borrow::Cow};
use std::env::args;
use regex::Regex;



#[derive(Debug)]
pub struct Line {
    pub line: u32,
    pub text: String,
    pub new_paragraph: bool,
    pub speaker: String,
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
            // println!("The Operation operator: {}, operand: {:?}", operator, operands);
            match (operator.as_str(), operands.as_slice()) {
                ("BT", _) => {
                    // println!("Begin Text: Clear previous.");
                    // Clear all prior state because we've just seen a
                    // "begin text" op
                    last_coords = None;
                    last_text = None;
                }
                ("Td", [Primitive::Number(x), Primitive::Number(y)]) => {
                    // "Text Location" contains the location of the text on the
                    // current page.
                    // println!("Set last coords:");
                    last_coords = Some((*x, *y));
                }
                ("Tm", _) => {
                    // println!("The Operation operator: {}, operand: {:?}", operator, operands);
                    // print_type_of(operands);
                    // println!("Operands Length: {}", operands.len());
                    // "Text Location" contains the location of the text on the
                    // current page.
                    // println!("Set last coords: ");

                    // for op in operands {
                    //     print_type_of(op);
                    //     let x = op.as_number().or(0.0)
                    // }

                    if operands.len() == 6{
                        last_coords = Some((operands[4].as_number().unwrap(), operands[5].as_number().unwrap()))
                    }

                    // last_coords = Some((*x, *y));
                }
                ("Tj", [Primitive::String(text)]) => {
                    // println!("The Operation operator: {}, operand: {:?}", operator, operands);
                    // print_type_of(operands);

                    // println!("Tj Setting text: {:?}", text.as_str().ok());
                    // "Show text" - the operation that actually contains the
                    // text to be displayed.
                    last_text = text.as_str().ok();
                    
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
                  
                    // let joined = a.join("");
                    // let _string = a.iter().map(|x| x.as_str()).collect();
             
                    // "Show text" - the operation that actually contains the
                    // text to be displayed.
                    
                }
                ("ET", _) => {
                    // println!("End of Text");
                    // println!("Last Coordinates: {:?}", last_coords);
                    // "end of text" - we should have finished this text object,
                    // if we got all the right information then we can yield it
                    // to the caller. Otherwise, use take() to clear anything
                    // we've seen so far and continue.
                    if let (Some((x, y)), Some(text)) = (last_coords.take(), last_text.take()) {
                        // println!("Yield TextObject TextObject: {{x: {}, y: {}, text: {} }}", x, y, text);
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


fn parse_text_objects_on_page(page: &Page, page_num: usize) -> Result<Vec<Line>, Error> {

    let content = match &page.contents {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };
    println!("Get text objects for Page");

    let mut text_objects = text_objects(&content.operations).collect::<Vec<TextObject>>();
   
    // Sort all text objects from top to bottom and then from left to right
    text_objects.sort_by(
        |a, b| 
        b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal)
    .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)));
   
    println!("Finished - Get text objects");
    // println!("Text Objects: {:?}", text_objects)

    let mut info = vec![];

    let mut current_line_number = 0_u32;
    let mut objects_for_line: Vec<TextObject> = Vec::new();
    let mut new_paragraph = false;
    let mut speaker = String::from("UNKNOWN");
    let re = Regex::new(r"^[A-Z \s \.]+:").unwrap();

    for text_obj in text_objects {
        
        // println!("Text objects: {:?}", text_obj);
        const MARGIN_X_LEFT: f32 = 112.0;
        const INDENT_X_LEFT: f32 = 200.0;

        // println!("Number of Objects on Line {}: {:?}", current_line_number, objects_for_line.len());
      
        if text_obj.x < MARGIN_X_LEFT {
            // This is a new line number on the side

            // println!("Line number {}", text_obj.text);
            let r = text_obj.text.parse();
            match r {

                // New Line Number
                Ok(v) => {
                    let mut s = String::from("");

                    // if &objects_for_line.len() > &0 {
                    //     println!("NEW PARAGRAPH: {:?}", objects_for_line[0]);
                    //     if objects_for_line[0].x < INDENT_X_LEFT {
                    //         // This is a new paragraph
                    //     // println!("NEW PARAGRAPH: {:?}", objects_for_line[0]);
                         
                    //         new_paragraph = true;
                    //     } else {
                    //         println!("NOT NEW PARAGRAPH: {:?}", objects_for_line[0]);
                    //         new_paragraph = false;
                    //     }
                    // }

         
                    println!("NEW GROUP");
                    for o in &objects_for_line {

                        println!("{:?}", o);
                       
                        s = s + &o.text.to_string();

                        for cap in re.captures_iter(&s) {
                            // println!("Speaker {}", &cap[0]);
                            speaker = cap[0].replace(":", "");
                        }
                       
                    }

                   
                  
                    let full_line = Line {
                        line: current_line_number,
                        text: s,
                        new_paragraph: new_paragraph,
                        speaker: speaker.to_string(),
                    };
                    // Save the new line
                    info.push(full_line);
                    // Empty the variable to start new line
                    objects_for_line.clear();
                    // Set the current line number
                    current_line_number = v;
                },
                // Sometimes there is a blank space " " that is not parsable
                // When this happens just leave the line number alone.
                Err(_) => {},
            }
        } else {
            objects_for_line.push(text_obj)
        }
        
    }

    // Deal with the last line
    let mut s = String::from("");
    for o in &objects_for_line {
        const BOTTOM_Y_CUTTOFF: f32 = 27.0; 
        // remove page number
        // println!("{:?}", o);
        if o.y > BOTTOM_Y_CUTTOFF {
            s = s + &o.text.to_string()
        }
    
    }
    let full_line = Line {
        line: current_line_number,
        text: s,
        new_paragraph: new_paragraph,
        speaker: speaker
    };
    // Save the new line
    info.push(full_line);

    println!("Finished Sorting Text Objects for Page");
  
    Ok(info)
}


pub fn parse_pdf_transcript(pdf: File<Vec<u8>>) -> Result<(), Error> {
    // Parse the complete pdf

    let mut lines = Vec::<Line>::new();

    for (i, page) in pdf.pages().enumerate(){
        // println!("Processing page: {}", i);
        let page = page?;
        let lines_on_page = parse_text_objects_on_page(&page, i)
        .with_context(|| format!("Unable to parse the members on page {}", i + 1))?;

        for l in lines_on_page {
            if l.new_paragraph {
                println!("Speaker: {} Ln: {}: [NEW PARA.]{}", l.speaker, l.line, l.text);
            } else {
                println!("Speaker: {} Ln: {}: {}", l.speaker, l.line, l.text);
            }
            
        }

        // lines.extend(lines_on_page)
    }

    Ok(())
}