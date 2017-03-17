use std::path::Path;
use core::token::{ascii, TokenKind, Token, Source};
use util::BinarySearch;

#[derive(Debug)]
pub struct SourceReader<'a>(pub &'a [u8], pub &'a Path, pub usize, pub Vec<[usize; 3]>);

impl<'a> Source for SourceReader<'a> {
    fn line(&self, offset: usize) -> usize {
        if let Some(index) = self.find_line_index(offset) {
            return self.3[index][2];
        } else if self.0.len() > 0 {
            if self.3.is_empty() {
                return 1;
            } else if offset < self.0.len() {
                return self.3[self.3.len() - 1][2] + 1;
            }
        }
        return 0;
    }

    fn column(&self, offset: usize) -> usize {
        if let Some(index) = self.find_line_index(offset) {
            return offset - self.3[index][1];
        } else if self.0.len() > 0 {
            if self.3.is_empty() {
                return offset + 1;
            } else if offset < self.0.len() {
                return offset - self.3[self.3.len() - 1][0];
            }
        }
        return 0;
    }

    fn filename(&self) -> &Path {
        unimplemented!()
    }

    fn content(&self, tok: &Token) -> &[u8] {
        &self.0[tok.1..tok.2]
    }

    fn source(&self) -> &[u8] {
        unimplemented!()
    }

    fn get(offset: usize) -> u8 {
        unimplemented!()
    }
}

impl<'a> SourceReader<'a> {
    pub fn current(&self) -> u8 {
        if self.2 >= self.0.len() {
            return ascii::EOF;
        }
        return self.0[self.2];
    }

    /// 判断是否可向前。
    pub fn can_forward(&self) -> bool {
        self.2 < self.0.len()
    }

    /// 当前偏移位置+1，并处理行标和列标。
    pub fn forward(&mut self) -> bool {
        //        if !self.can_forward() {
        //            return false;
        //        }
        self.2 += 1;
        if self.current() == ascii::CR {
            //add line
            {
                let offs = self.2;
                if self.3.is_empty() {
                    self.3.push([offs, 0, 1]);
                } else {
                    if let None = self.find_line_index(offs) {
                        let arr = self.3[self.3.len() - 1];
                        self.3.push([offs, arr[0], arr[2] + 1]);
                    }
                }
            }
            // self.2 += 1; //吃掉回车？
        }
        return true;
    }

    /// 当前偏移位置-1，并处理行标和列标。
    pub fn back(&mut self) {
        if self.2 - 1 < 0 {
            panic!("超出索引");
        }
        self.2 -= 1;
    }

    /// 当前偏移位置+n, n为负数则调用 back() 否则 forward().
    pub fn seek(&mut self, n: isize) {
        if n < 0 {
            for i in 0..n.abs() {
                self.back()
            }
        } else if n > 0 {
            for i in 0..n {
                self.forward();
            }
        }
    }

    /// 根据当前位置与参数 pos 的差值调用 back() 。
    pub fn back_pos_diff(&mut self, pos: usize) {
        if pos >= self.2 {
            return;
        }
        let n = (self.2 - pos) as isize;
        self.seek(-n);
    }

    pub fn offset(&mut self) -> usize {
        let offset = self.2;
        return offset.clone();
    }

    /// 与当前偏移的下一个字符作比较，如果可用的话。
    pub fn match_forward(&self, b: u8) -> bool {
        return self.match_forward_n(1, b);
    }

    /// 与当前偏移的下一个字符作比较，如果可用的话。
    pub fn match_forward_n(&self, n: usize, b: u8) -> bool {
        if self.2 + n < 0 || self.2 + n >= self.0.len() {
            return false;
        }
        return self.0[self.2 + n] == b;
    }

    pub fn find_line_index(&self, offset: usize) -> Option<usize> {
        return self.3.binary_search(Box::new(move |item: &[usize; 3]| -> isize{
            if item[1] >= offset && offset <= item[0] {
                return 0;
            } else if item[0] > offset { return 1; }
            return -1;
        }));
    }
}