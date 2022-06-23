#[derive(PartialEq)]
pub enum LoispDatatype {
    Integer,
    String,
    Word,
    Nothing
}

impl LoispDatatype {
    pub fn size(&self) -> usize {
        match self {
            Self::Integer => 8,
            Self::String => 8,
            Self::Word => 0,
            Self::Nothing => 0
        }
    }
}
