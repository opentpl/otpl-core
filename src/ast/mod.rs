mod visitor;

pub use self::visitor::*;
use token::Token;

/// 定义的一个语法树的节点集合。
pub type NodeList = Vec<Node>;

/// 定义的一个语法树的分类抽象节点。
#[derive(Debug, Clone)]
pub enum Node {
    /// 表示一个用于占位的空节点，它不产生任何副作用。
    Empty,
    /// 表示语法树的根。
    Root(NodeList),
    /// 表示一个字面量。
    Literal(Token),
    /// 表是一个 DOM 标签节点，如：div。
    DomTag(Token, Vec<DomAttr>, NodeList),
    /// 表示一个集合。
    List(NodeList),
    /// 代码段
    Statement(NodeList),
    /// 三目表达式(express,left,right)
    Ternary(Box<Node>, Box<Node>, Box<Node>),
    /// 二元表达式(left,right,operator)
    Binary(Box<Node>, Box<Node>, Operator),
    /// 一元表达式(body,operator）
    Unary(Box<Node>, Operator),
    /// 访问成员属性(object, parameters, operator)
    Property(Box<Node>, NodeList, Token),
    /// 访问成员方法(object, parameters, operator)
    Method(Box<Node>, NodeList, Token),
    /// 表示一个标示符，如：变量名。
    Identifier(Token),
    /// if/else-if条件表达式(condition, body, branch-blocks,is-else-if)
    If(Box<Node>, NodeList, NodeList, bool),
    /// else表达式(body)
    Else(NodeList),
    /// for表达式(key-name, value-name, iter, body,else)
    For(Token, Token, Box<Node>, NodeList, Box<Node>),
    Print(Box<Node>, bool),
    /// 表示一个常量
    Const(Constant),
}

/// 表示一个 DOM 节点的属性，如： id。
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Constant {
    Break,
    Continue,
    None,
    True,
    False,
    /// 表示一个字符串。
    String(Token),
    /// 表示一个无符号整数常量。
    Integer(Token),
    /// 表示一个无符号浮点数常量(integer,decimal)。
    Float(Token, Token),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operator {
    /// + or +x
    Add,
    /// - or -x
    Sub,
    /// *
    Mul,
    /// /
    Div,
    /// %
    Mod,
    /// >
    Gt,
    /// >=
    Gte,
    /// <
    Lt,
    /// <=
    Lte,
    /// ==
    Eq,
    /// !=
    NotEq,
    /// &&
    And,
    /// ||
    Or,
    /// ??
    NullCond,
    /// ?
    TestCond,

}
