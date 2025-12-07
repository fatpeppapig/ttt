#[derive(Clone, Copy, Debug)]
pub struct Glyph {
    pub ch: char,
    pub idx: usize,
}

pub type Layout = Vec<Vec<Glyph>>;

pub enum TextSource {
    RandomWords(Vec<String>),
    Fixed(String),
}
