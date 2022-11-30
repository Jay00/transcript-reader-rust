
pub enum Gender {
    Female,
    Male,
}

#[derive(Debug)]
pub struct Speaker {
    pub name: String,
    pub gender: Gender,
}


#[derive(Debug)]
pub struct Paragraph {
    pub lines: Line,
}

#[derive(Debug)]
pub struct Statement {
    pub speaker: Speaker,
    pub paragraphs: Vec<Paragraph>,
    pub is_question: bool,
}

#[derive(Debug)]
pub struct Transcript {
    pub sections: Vec<Statement>,
}
