use super::{Node, Root, NodeList, DomTag};
use token::Token;

/// 定义一个用于访问AST节点的一组方法。
pub trait Visitor {
    /// 访问分类抽象节点。
    fn visit(&mut self, node: &Node) {
        match node {
            &Node::Root(ref inner) => self.visit_root(inner),
            &Node::Literal(ref inner) => self.visit_literal(inner),
            &Node::DomTag(ref inner) => self.visit_dom_tag(inner),
            &Node::Ternary(ref node,ref left,ref right) => self.visit_ternary(node.as_ref(),left.as_ref(),right.as_ref()),
            _ => self.visit_undefined(node)
        }
    }
    /// 访问未在本访问器定义的 Node。
    #[allow(unused_variables)]
    fn visit_undefined(&mut self, node: &Node) {
        match node {
            &Node::Empty => {}
            _ => println!("warning: undefined visit node {:?}", node)
        }
    }
    fn visit_root(&mut self, root: &Root) {
        for n in &root.body {
            self.visit(&n);
        }
    }
    fn visit_list(&mut self, list: &NodeList) {
        for n in list {
            self.visit(&n);
        }
    }
    /// 访问字面量
    fn visit_literal(&mut self, tok: &Token);
    /// 访问 DomTag
    fn visit_dom_tag(&mut self, tag: &DomTag);
    fn visit_ternary(&mut self, node: &Node, left: &Node, right: &Node);
}