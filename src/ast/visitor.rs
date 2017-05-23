use super::{Node, NodeList, DomAttr};
use token::Token;

/// 定义一个用于访问AST节点的一组方法。
pub trait Visitor {
    /// 访问分类抽象节点。
    fn visit(&mut self, node: &Node) {
        match node {
            &Node::Root(ref inner) => self.visit_root(inner),
            &Node::Literal(ref inner) => self.visit_literal(inner),
            &Node::DomTag(ref name,ref attrs,ref children) => self.visit_dom_tag(name,attrs,children),
            &Node::Statement(ref inner) => self.visit_statement(inner),
            &Node::Ternary(ref expr,ref left,ref right) => self.visit_ternary(expr,left,right),
            &Node::Binary(ref left,ref right,ref operator) => self.visit_binary(left,right,operator),
            &Node::Unary(ref body,ref operator) => self.visit_unary(body,operator),
            &Node::Property(ref obj,ref params,ref operator) => self.visit_property(obj,params,operator),
            &Node::Method(ref obj,ref params,ref operator) => self.visit_method(obj,params,operator),
            &Node::String(ref inner) => self.visit_string(inner),
            &Node::Boolean(ref inner) => self.visit_boolean(inner),
            &Node::Integer(ref inner) => self.visit_integer(inner),
            &Node::Float(ref integer,ref decimal) => self.visit_float(integer,decimal),
            &Node::None(ref inner) => self.visit_none(inner),
            &Node::Identifier(ref inner) => self.visit_identifier(inner),
            &Node::If(ref condition,ref body,ref branches,ref is_else_if) => self.visit_if(condition,body,branches,is_else_if),
            &Node::Else(ref body) => self.visit_else(body),
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
    fn visit_root(&mut self, body: &NodeList) {
        self.visit_list(body);
    }
    fn visit_list(&mut self, list: &NodeList) {
        for n in list {
            self.visit(&n);
        }
    }
    /// 访问字面量
    fn visit_literal(&mut self, tok: &Token);
    /// 访问 DomTag
    fn visit_dom_tag(&mut self, name: &Token, attrs: &Vec<DomAttr>,children:&NodeList);
    fn visit_statement(&mut self, body: &NodeList) {
        self.visit_list(body);
    }
    fn visit_ternary(&mut self, expr: &Node,left: &Node, right: &Node);
    fn visit_binary(&mut self, left: &Node, right: &Node,operator: &Token);
    fn visit_unary(&mut self, body: &Node,operator: &Token);
    fn visit_property(&mut self, obj: &Node, params: &NodeList,operator: &Token);
    fn visit_method(&mut self, obj: &Node, params: &NodeList,operator: &Token);
    fn visit_string(&mut self, tok: &Token);
    fn visit_boolean(&mut self, tok: &Token);
    fn visit_integer(&mut self, tok: &Token);
    fn visit_float(&mut self, integer: &Token, decimal: &Token);
    fn visit_none(&mut self, tok: &Token);
    fn visit_identifier(&mut self, tok: &Token);
    fn visit_if(&mut self, condition: &Node,body: &NodeList,branches: &NodeList,is_else_if:&bool);
    fn visit_else(&mut self,body: &NodeList){
        self.visit_list(body);
    }
}