use super::Node;
use super::NodeList;
use super::token::Token;

/// 表示一个 DOM 节点的标签，如： div。
#[derive(Debug)]
pub struct DomTag<'a> {
    pub name: Token<'a>,
    pub attrs: Vec<DomAttr<'a>>,
    pub children: NodeList<'a>,
}

impl <'a> DomTag<'a>{
    pub fn new(name: Token<'a>) -> DomTag<'a> {
        return DomTag{
            name: name,
            attrs: vec![],
            children: vec![],
        };
    }
}

/// 表示一个 DOM 节点的属性，如： id。
#[derive(Debug)]
pub struct DomAttr<'a> {
    pub name: Token<'a>,
    pub value: NodeList<'a>,
}

impl <'a> DomAttr<'a>{
    pub fn new(name: Token<'a>) -> DomAttr<'a> {
        return DomAttr{
            name: name,
            value: vec![],
        };
    }
}