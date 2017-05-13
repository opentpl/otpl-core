mod dom;
mod visitor;

pub use self::dom::*;
pub use self::visitor::*;
use token::Token;

//https://www.oschina.net/question/81620_239264


/// 定义的一个语法树的节点集合。
pub type NodeList = Vec<Node>;

/// 定义的一个语法树的分类抽象节点。
#[derive(Debug)]
pub enum Node {
    /// 表是一个用于占位的空节点。
    Empty,
    Literal(Token),
    Root(Root),
    /// 表是一个 DOM 节点，如：div。
    DomTag(DomTag),
    /// 代码段
    Statement(NodeList),
    /// 三目表达式（express,left,right）
    Ternary(Box<Node>,Box<Node>,Box<Node>),
    /// 二元表达式（left,right,operator）
    Binary(Box<Node>,Box<Node>,Token),
    /// 一元表达式（body,operator）
    Unary(Box<Node>,Token),
    /// 成员属性（object, parameters, operator）
    Property(Box<Node>,NodeList,Token),
    /// 成员方法（object, parameters, operator）
    Method(Box<Node>,NodeList,Token),
    String(Token),
    Boolean(Token),
    Integer(Token),
    Float(Token,Token),
    None(Token),
    Identifier(Token),
    List(NodeList),

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

#[derive(Debug)]
pub struct TernaryNode(pub Node,pub Node,pub Node);