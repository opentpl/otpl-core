use super::*;

/// 定义一个用于访问AST节点的一组方法。
pub trait Visitor {

    /// 访问分类抽象节点。
    fn visit(&mut self, node: &Node) {
        match node {
            &Node::DomNode(ref inner) => self.visit_dom_node(inner),
            _ => self.visit_undefined(node)
        }
    }
    /// 访问未在本访问器定义的 Node。
    #[allow(unused_variables)]
    fn visit_undefined(&mut self, node: &Node){
        match node {
            &Node::None  => {},
            _ => println!("warning: undefined visit node {:?}", node)
        }
    }
    /// 访问 DomNode
    fn visit_dom_node(&mut self, node: &DomNode);
}