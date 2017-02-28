use std::fmt::Debug;

/// 节点类型。
#[derive(Debug)]
pub enum NodeType {
    None,
    DomNode,
    DomAttr,
}

/// 定义的一个语法树的节点。
pub trait Node: Debug {
    fn kind(&self)      -> NodeType;
    fn line(&self)      -> i32;
    fn column(&self)    -> i32;
}

/// 表是一个 DOM 节点，如：div。
#[allow(dead_code)]
#[derive(Debug)]
pub struct DomNode<'a> {
    line    : i32,
    column  : i32,
    attrs   : Vec<DomAttr<'a>>,
    children: Vec<Box<Node>>,
}

impl<'a> Node for DomNode<'a> {
    fn line(&self)      -> i32 { self.line }
    fn column(&self)    -> i32 { self.column }
    fn kind(&self)      -> NodeType { NodeType::DomNode }
}

/// 表示一个 DOM 节点的属性，如： id。
#[derive(Debug)]
pub struct DomAttr<'a> {
    line    : i32,
    column  : i32,
    name    : &'a str,
    value   : Box<Node>,
}

impl<'a> Node for DomAttr<'a> {
    fn line(&self)      -> i32 { self.line }
    fn column(&self)    -> i32 { self.column }
    fn kind(&self)      -> NodeType { NodeType::DomAttr }
}

/// 用于替换 Option 的一个空的节点。
#[derive(Debug)]
pub struct NoneNode;
impl Node for NoneNode {
    fn line(&self)      -> i32 { 0 }
    fn column(&self)    -> i32 { 0 }
    fn kind(&self)      -> NodeType { NodeType::None }
}

pub fn buildAST() -> Box<Node> {
    let mut tag = DomNode { line: 0, column: 0, children: vec![], attrs: vec![] };
    tag.attrs.push(DomAttr { line: 0, column: 0, name: "id", value: Box::new(NoneNode{}) });
    return Box::new(tag);
}


#[test]
fn it_works2() {
        let node = buildAST();
        //let attr: &mut DomAttr = node.as_ref();
        match node.kind() {
            NodeType::DomNode   => println!("aaa"),
            _                   => println!("bbb"),
        }
    println!("1======> {:?}", node);
}