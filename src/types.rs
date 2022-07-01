#[derive(PartialEq)]
pub enum LoispDatatype {
    Integer,
    String,
    Word,
    Nothing,
    Pointer,
}

impl LoispDatatype {
    pub fn size(&self) -> usize {
        match self {
            Self::Integer => 8,
            Self::String => 8,
            Self::Word => 0,
            Self::Nothing => 0,
            Self::Pointer => 8,
        }
    }
}
