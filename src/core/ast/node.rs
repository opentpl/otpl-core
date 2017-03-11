use super::DomTag;
use super::token::Token;
//https://www.oschina.net/question/81620_239264


/// 定义的一个语法树的节点集合。
pub type NodeList<'a> = Vec<Node<'a>>;

/// 定义的一个语法树的分类抽象节点。
#[derive(Debug)]
pub enum Node<'a> {
    /// 表是一个用于占位的空节点。
    None,
    Literal(Token<'a>),
    Root(Root<'a>),
    /// 表是一个 DOM 节点，如：div。
    DomTag(DomTag<'a>),
}

#[derive(Debug)]
pub struct Root<'a> {
    pub body: NodeList<'a>,
}

impl<'a> Root<'a> {
    pub fn new() -> Root<'a> {
        return Root {
            body: vec![],
        };
    }
}
