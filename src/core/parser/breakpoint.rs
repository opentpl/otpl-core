use super::Parser;
use core::token::Token;
use util::VecSliceCompare;
use util::Queue;
pub struct BreakPoint {
    /// 是否保留已检查的token
    pub keep: bool,
    pub values: Vec<Vec<u8>>,
}

impl BreakPoint {
    pub fn new(keep: bool, values: Vec<Vec<u8>>) -> BreakPoint {
        BreakPoint { keep: keep, values: values }
    }


    pub fn build(breaks: Vec<BreakPoint>) -> Box<(FnMut(&mut Parser) -> bool)> {
        return Box::new(move |owner: &mut Parser| -> bool {
//            let breaks: Vec<BreakPoint> = vec![
//                BreakPoint::new(false, vec![vec![ascii::SLA], tag_name.clone()]),
//            ];

            let mut found;
            for point in &breaks {
                if point.values.is_empty() {
                    continue;
                }
                found = true;
                let mut buf: Vec<Token> = vec![];
                for value in &point.values {
                    if let Option::Some(tok) = owner.take() {
                        if !value.compare(tok.str) {
                            found = false;
                        }
                        buf.push(tok);
                    } else {
                        found = false;
                    }
                    if !found {break;}
                }
                if !found || point.keep {
                    while !buf.is_empty() {
                        owner.back(buf.pop().unwrap());
                    }
                }

                if found {
                    return found;
                }
            }
            return false;
        });
    }

}