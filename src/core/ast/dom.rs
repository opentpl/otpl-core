use super::Node;
use super::NodeList;
use super::token::Token;

/// 定义的一个语法树的节点。
#[derive(Debug)]
pub struct DomNode<'a> {
    pub pos: Token<'a>,
    pub attrs: Vec<DomAttr<'a>>,
    pub children: NodeList<'a>,
}

impl <'a> DomNode<'a>{
    pub fn new(pos: Token<'a>) -> DomNode<'a> {
        return DomNode{
            pos: pos,
            attrs: vec![],
            children: vec![],
        };
    }
}

/// 表示一个 DOM 节点的属性，如： id。
#[derive(Debug)]
pub struct DomAttr<'a> {
    pub pos: Token<'a>,
    pub value: NodeList<'a>,
}

impl <'a> DomAttr<'a>{
    pub fn new(pos: Token<'a>) -> DomAttr<'a> {
        return DomAttr{
            pos: pos,
            value: vec![],
        };
    }
}