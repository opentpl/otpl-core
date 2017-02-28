use super::Node;
use super::NodeList;

/// 定义的一个语法树的节点。
#[derive(Debug)]
pub struct DomNode<'a> {
    pub line: i32,
    pub column: i32,
    pub tag: &'a str,
    pub attrs: Vec<DomAttr<'a>>,
    pub children: NodeList<'a>,
}

impl <'a> DomNode<'a>{
    pub fn new(line: i32, column: i32, tag: &'a str) -> DomNode {
        return DomNode{
            line: line,
            column: column,
            tag: tag,
            attrs: vec![],
            children: vec![],
        };
    }
}

/// 表示一个 DOM 节点的属性，如： id。
#[derive(Debug)]
pub struct DomAttr<'a> {
    pub line: i32,
    pub column: i32,
    pub name: &'a str,
    pub value: Node<'a>,
}

impl <'a> DomAttr<'a>{
    pub fn new(line: i32, column: i32, name: &'a str, value: Node<'a>) -> DomAttr<'a> {
        return DomAttr{
            line: line,
            column: column,
            name: name,
            value: value,
        };
    }
}