pub mod ascii;

/// 用于记录节点位于原代码中的位置，展开(lineNo, column, srcSlice)。
#[derive(Debug, Clone)]
pub struct Pos<'a> {
    pub line: usize,
    pub column: usize,
    pub str: &'a [u8],
}

impl<'a> Pos<'a> {
    pub fn new(line: usize, column: usize, str: &'a [u8]) -> Pos<'a> {
        Pos { line: line, column: column, str: str }
    }
}

//#[derive(Debug, Clone)]
//pub enum Token<'a> {
//    None,
//    LSS(usize, usize),
//    LEQ(usize, usize),
//    GTR(usize, usize),
//    GEQ(usize, usize),
//    Symbol(usize, usize, u8),
//    Data(usize, usize, Vec<u8>),
//    StmtStart(usize, usize, Vec<u8>),
//    StmtEnd(usize, usize, Vec<u8>),
//    LiteralBoundary(usize, usize, bool),
//    Literal(usize, usize, Vec<u8>),
//    Comments(usize, usize, Vec<u8>),
//    DomTagStart(Pos<'a>),
//    DomTagEnd(Pos<'a>),
//    DomTagAttrStart(Pos<'a>),
//    DomTagAttrEnd(Pos<'a>),
//}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenKind {
    Symbol,
    DomTagStart,
    DomTagEnd,
    DomTagAttrStart,
    DomTagAttrEnd,
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub line: usize,
    pub column: usize,
    pub str: &'a [u8],
    pub kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn new(line: usize, column: usize, str: &'a [u8], kind: TokenKind) -> Token<'a> {
        Token { line: line, column: column, str: str , kind: kind}
    }
}
#[test]
fn kk(){
    //let x = TokenKind::DomTagStart|TokenKind::DomTagStart;

}