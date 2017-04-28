use token::Token;
use super::NodeList;

/// 表示一个 DOM 节点的标签，如： div。
#[derive(Debug)]
pub struct DomTag {
    pub name: Token,
    pub attrs: Vec<DomAttr>,
    pub children: NodeList,
}

impl DomTag {
    pub fn new(name: Token) -> DomTag {
        return DomTag {
            name: name,
            attrs: vec![],
            children: vec![],
        };
    }
}

/// 表示一个 DOM 节点的属性，如： id。
#[derive(Debug)]
pub struct DomAttr {
    pub name: Token,
    pub value: NodeList,
}

impl DomAttr {
    pub fn new(name: Token) -> DomAttr {
        return DomAttr {
            name: name,
            value: vec![],
        };
    }
}