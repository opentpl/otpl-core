pub mod ascii;

#[derive(Debug)]
pub enum Token {
    None,
    LSS(usize, usize),
    LEQ(usize, usize),
    GTR(usize, usize),
    GEQ(usize, usize),
    Symbol(usize, usize, u8),
    Data(usize, usize, Vec<u8>),
    StmtStart(usize, usize, Vec<u8>),
    StmtEnd(usize, usize, Vec<u8>),
    LiteralBoundary(usize, usize, bool),
    Literal(usize, usize, Vec<u8>),
    Comments(usize, usize, Vec<u8>),
    DomTagStart(usize, usize, Vec<u8>),
    DomTagAttrName(usize, usize, Vec<u8>),
    DomTagEnd(usize, usize, Vec<u8>),

}