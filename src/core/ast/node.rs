use super::DomTag;
use super::token::Token;
//https://www.oschina.net/question/81620_239264


/// 定义的一个语法树的节点集合。
pub type NodeList = Vec<Node>;

/// 定义的一个语法树的分类抽象节点。
#[derive(Debug)]
pub enum Node {
    /// 表是一个用于占位的空节点。
    None,
    Literal(Token),
    Root(Root),
    /// 表是一个 DOM 节点，如：div。
    DomTag(DomTag),
}

#[derive(Debug)]
pub struct Root {
    pub body: NodeList,
}

impl Root {
    pub fn new() -> Root {
        return Root {
            body: vec![],
        };
    }
}
